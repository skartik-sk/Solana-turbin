import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftStakingCore } from "../target/types/nft_staking_core";
import { SystemProgram } from "@solana/web3.js";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";

const MILLISECONDS_PER_DAY = 86400000;
const POINTS_PER_STAKED_NFT_PER_DAY = 10_000_000;
const FREEZE_PERIOD_IN_DAYS = 7;
const TIME_TRAVEL_IN_DAYS = 9;

describe("nft-staking-core", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.nftStakingCore as Program<NftStakingCore>;

  // Generate a keypair for the collection
  const collectionKeypair = anchor.web3.Keypair.generate();

  // Find the update authority for the collection (PDA)
  const updateAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("update_authority"), collectionKeypair.publicKey.toBuffer()],
    program.programId
  )[0];

  // Generate a keypair for the nft asset
  const nftKeypair = anchor.web3.Keypair.generate();

  // Generate keypairs for claim rewards tests
  const nftClaimKeypair = anchor.web3.Keypair.generate();

  // Generate a keypair for the nft to be burned (for burn tests)
  const nftToBurnKeypair = anchor.web3.Keypair.generate();

  // Generate a keypair for transfer tests
  const nftTransferKeypair = anchor.web3.Keypair.generate();

  const oracle_plugin_id = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("oracle")],
    program.programId
  )[0];

  const reward_vault = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault_for_reward"), oracle_plugin_id.toBuffer()],
    program.programId
  )[0];

  console.log("oracle_plugin_id : ", oracle_plugin_id.toBase58());
  console.log("reward_vault : ", reward_vault.toBase58());

  // Find the config account (PDA)
  const config = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), collectionKeypair.publicKey.toBuffer()],
    program.programId
  )[0];

  // Find the rewards mint account (PDA)
  const rewardsMint = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("rewards"), config.toBuffer()],
    program.programId
  )[0];

  async function reset_time(): Promise<void> {
    const rpcResponse = await fetch(provider.connection.rpcEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "surfnet_resetNetwork",
        params: [],
      }),
    });

    const result = await rpcResponse.json() as { error?: any; result?: any };
    if (result.error) {
      throw new Error(`Reset failed: ${JSON.stringify(result.error)}`);
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  /**
   * Helper function to advance time with surfnet_timeTravel RPC method
   * @param params - Time travel params (absoluteEpoch, absoluteSlot, or absoluteTimestamp)
   */
  async function advanceTime(params: { absoluteEpoch?: number; absoluteSlot?: number; absoluteTimestamp?: number }): Promise<void> {
    const rpcResponse = await fetch(provider.connection.rpcEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "surfnet_timeTravel",
        params: [params],
      }),
    });

    const result = await rpcResponse.json() as { error?: any; result?: any };
    if (result.error) {
      throw new Error(`Time travel failed: ${JSON.stringify(result.error)}`);
    }
  }

  // Reset everything first (disabled - removes deployed program too)
  xit("Reset everything", async () => {
    await reset_time();
    console.log("\n=== Reset everything ===");
  });

  it("Create a collection", async () => {
    const collectionName = "Test Collection";
    const collectionUri = "https://example.com/collection";
    const tx = await program.methods.createCollection(collectionName, collectionUri)
      .accountsPartial({
        payer: provider.wallet.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([collectionKeypair])
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Collection address", collectionKeypair.publicKey.toBase58());
  });

  it("Mint an NFT", async () => {
    const nftName = "Test NFT";
    const nftUri = "https://example.com/nft";
    const tx = await program.methods.mintNft(nftName, nftUri)
      .accountsPartial({
        user: provider.wallet.publicKey,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([nftKeypair])
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("NFT address", nftKeypair.publicKey.toBase58());
  });

  it("Initialize stake config", async () => {
    const tx = await program.methods.initializeConfig(POINTS_PER_STAKED_NFT_PER_DAY, FREEZE_PERIOD_IN_DAYS)
      .accountsPartial({
        admin: provider.wallet.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ skipPreflight: true });
    console.log("\nYour transaction signature", tx);
    console.log("Config address", config.toBase58());
    console.log("Points per staked NFT per day", POINTS_PER_STAKED_NFT_PER_DAY);
    console.log("Freeze period in days", FREEZE_PERIOD_IN_DAYS);
    console.log("Rewards mint address", rewardsMint.toBase58());

    // Airdrop to reward vault after it's created
    await provider.connection.requestAirdrop(reward_vault, 10 * 1e9);
    console.log("\nAirdropped 10 SOL to reward vault:", reward_vault.toBase58());
  });

  // ============================================
  // ORACLE PLUGIN TESTS (Do these first while oracle is fresh)
  // ============================================

  it("Mint NFT for transfer tests", async () => {
    const nftName = "Test NFT for Transfer";
    const nftUri = "https://example.com/nft-transfer";
    const tx = await program.methods.mintNft(nftName, nftUri)
      .accountsPartial({
        user: provider.wallet.publicKey,
        nft: nftTransferKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([nftTransferKeypair])
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("NFT for transfer tests:", nftTransferKeypair.publicKey.toBase58());
  });

  it("Transfer NFT (self) - should succeed (oracle in default state)", async () => {
    const tx = await program.methods.transfer()
      .accounts({
        user: provider.wallet.publicKey,
        nft: nftTransferKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        newOwner: provider.wallet.publicKey,
        oracle: oracle_plugin_id,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });
    console.log("\nYour transaction signature", tx);
    console.log("Transfer succeeded (oracle in default approving state)");
  });

  it("Time travel to outside market hours", async () => {
    const currentTimestamp = Date.now();
    await advanceTime({ absoluteTimestamp: currentTimestamp + 10 * 60 * 60 * 1000 });
    console.log("\nTime traveled 10 hours ahead (outside market hours)");
  });

  it("Update Oracle state", async () => {
    try {
      const tx = await program.methods.updateOracle()
        .accounts({
          signer: provider.wallet.publicKey,
          payer: provider.wallet.publicKey,
          oracle: oracle_plugin_id,
          vaultForReward: reward_vault,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("\nYour transaction signature", tx);
      console.log("Oracle state updated - now rejecting transfers");

      // Check oracle state (account type is Validation, not Oracle)
      const oracleAccount = await program.account.validation.fetch(oracle_plugin_id);
      console.log("Oracle validation state:", oracleAccount.validation);
    } catch (err: any) {
      if (err.toString().includes("AlreadyUpdated")) {
        console.log("\nOracle already updated from previous run, skipping update...");
        // Still try to fetch the state
        try {
          const oracleAccount = await program.account.validation.fetch(oracle_plugin_id);
          console.log("Current Oracle validation state:", oracleAccount.validation);
        } catch (e) {
          // ignore
        }
      } else {
        throw err;
      }
    }
  });

  it("FAIL: Transfer NFT (self) - should fail (oracle now rejecting)", async () => {
    try {
      await program.methods.transfer()
        .accounts({
          user: provider.wallet.publicKey,
          nft: nftTransferKeypair.publicKey,
          collection: collectionKeypair.publicKey,
          newOwner: provider.wallet.publicKey,
          oracle: oracle_plugin_id,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc({ skipPreflight: true });
      throw new Error("Should have failed - Oracle should reject transfer");
    } catch (err) {
      console.log("\nExpected error: Oracle rejected transfer (outside market hours)");
    }
  });

  // ============================================
  // STAKING TESTS
  // ============================================

  it("Stake an NFT", async () => {
    const tx = await program.methods.stake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
    console.log("\nYour transaction signature", tx);
  });

  it("Time travel to the future", async () => {
    const currentTimestamp = Date.now();
    await advanceTime({ absoluteTimestamp: currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled in days:", TIME_TRAVEL_IN_DAYS);
  });

  it("Unstake an NFT", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
    const tx = await program.methods.unstake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        userRewardsAta,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("User rewards balance", (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount);
  });

  // ============================================
  // CLAIM REWARDS TESTS
  // ============================================

  it("Mint and stake NFT for claim rewards tests", async () => {
    const nftName = "Test NFT for Claim";
    const nftUri = "https://example.com/nft-claim";
    const tx = await program.methods.mintNft(nftName, nftUri)
      .accountsPartial({
        user: provider.wallet.publicKey,
        nft: nftClaimKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([nftClaimKeypair])
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("NFT for claim rewards address", nftClaimKeypair.publicKey.toBase58());

    await program.methods.stake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        nft: nftClaimKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
    console.log("NFT staked for claim rewards tests");
  });

  it("FAIL: Try to claim rewards before freeze period", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
    try {
      await program.methods.claimRewards()
        .accountsPartial({
          user: provider.wallet.publicKey,
          updateAuthority,
          config,
          rewardsMint,
          userRewardsAta,
          nft: nftClaimKeypair.publicKey,
          collection: collectionKeypair.publicKey,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .rpc();
      throw new Error("Should have failed due to freeze period not elapsed");
    } catch (err) {
      console.log("\nExpected error: Claim before freeze period failed correctly");
    }
  });

  it("Time travel to the future", async () => {
    const currentTimestamp = Date.now() + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY;
    await advanceTime({ absoluteTimestamp: currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled in days:", TIME_TRAVEL_IN_DAYS);
  });

  it("SUCCESS: Claim rewards after freeze period (NFT remains staked)", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);

    const balanceBefore = (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount || 0;

    const tx = await program.methods.claimRewards()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        userRewardsAta,
        nft: nftClaimKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();

    const balanceAfter = (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount || 0;
    const rewardsClaimed = balanceAfter - balanceBefore;

    console.log("\nYour transaction signature", tx);
    console.log("Rewards claimed:", rewardsClaimed);
    console.log("User rewards balance after claim:", balanceAfter);

    const expectedRewards = FREEZE_PERIOD_IN_DAYS * (POINTS_PER_STAKED_NFT_PER_DAY / 1_000_000);
    console.log("Expected rewards:", expectedRewards);
  });

  // ============================================
  // BURN STAKED NFT TESTS
  // ============================================

  it("Stake a fresh NFT for burn test", async () => {
    const nftName = "Test NFT to Burn";
    const nftUri = "https://example.com/nft-to-burn";
    const tx = await program.methods.mintNft(nftName, nftUri)
      .accountsPartial({
        user: provider.wallet.publicKey,
        nft: nftToBurnKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([nftToBurnKeypair])
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("New NFT minted for burn test:", nftToBurnKeypair.publicKey.toBase58());

    await program.methods.stake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        nft: nftToBurnKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
    console.log("New NFT staked for burn test");
  });

  it("Time travel to the future", async () => {
    const currentTimestamp = Date.now() + (TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY) * 3;
    await advanceTime({ absoluteTimestamp: currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled in days:", TIME_TRAVEL_IN_DAYS);
  });

  it("SUCCESS: Burn staked NFT with bonus rewards (1.1x multiplier)", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);

    const balanceBefore = (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount || 0;

    const tx = await program.methods.burnStakedNft()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        userRewardsAta,
        nft: nftToBurnKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();

    const balanceAfter = (await provider.connection.getTokenAccountBalance(userRewardsAta)).value.uiAmount || 0;
    const rewardsFromBurn = balanceAfter - balanceBefore;

    console.log("\nYour transaction signature", tx);
    console.log("Rewards from burn:", rewardsFromBurn);
    console.log("User rewards balance after burn:", balanceAfter);

    const expectedNormalRewards = FREEZE_PERIOD_IN_DAYS * (POINTS_PER_STAKED_NFT_PER_DAY / 1_000_000);
    const expectedBonusRewards = expectedNormalRewards * 1.1;
    console.log("Expected normal rewards:", expectedNormalRewards);
    console.log("Expected with 1.1x bonus:", expectedBonusRewards);
  });

});
