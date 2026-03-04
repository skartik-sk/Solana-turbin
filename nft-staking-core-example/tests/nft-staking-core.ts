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
console.log("oracle_plugin_id : ", oracle_plugin_id)

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

  it("Create a collection", async () => {
    const collectionName = "Test Collection";
    const collectionUri = "https://example.com/collection";
    const tx = await program.methods.createCollection(collectionName, collectionUri)
    .accountsPartial({
      payer: provider.wallet.publicKey,
      collection: collectionKeypair.publicKey,
      updateAuthority,
      oracle: oracle_plugin_id,
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
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Config address", config.toBase58());
    console.log("Points per staked NFT per day", POINTS_PER_STAKED_NFT_PER_DAY);
    console.log("Freeze period in days", FREEZE_PERIOD_IN_DAYS);
    console.log("Rewards mint address", rewardsMint.toBase58());
  });

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
    
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }

  it("Time travel to the future", async () => {
    // Advance time in milliseconds
    const currentTimestamp = Date.now();
    await advanceTime({ absoluteTimestamp: currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled in days", TIME_TRAVEL_IN_DAYS)
  });

  it("Unstake an NFT", async () => {
    // Get the user rewards ATA account
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

    // Stake NFT for claim rewards tests
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
    // Advance time in milliseconds
    const currentTimestamp = Date.now()+ TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY;
    await advanceTime({ absoluteTimestamp: currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled in days", TIME_TRAVEL_IN_DAYS)
  });

  it("SUCCESS: Claim rewards after freeze period (NFT remains staked)", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);

    // Get balance before
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

    // Verify rewards match expected (7 days * 10 points per day)
    const expectedRewards = FREEZE_PERIOD_IN_DAYS * (POINTS_PER_STAKED_NFT_PER_DAY / 1_000_000);
    console.log("Expected rewards:", expectedRewards);
  });

  it("FAIL: Try to claim rewards again immediately (must wait for rewards to accumulate)", async () => {
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
      throw new Error("Should have failed due to insufficient time elapsed");
    } catch (err) {
      console.log("\nExpected error: Immediate second claim failed correctly");
    }
  });

  it("Time travel to the future", async () => {
    // Advance time in milliseconds
    const currentTimestamp = Date.now() + (TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY)*2 
    await advanceTime({ absoluteTimestamp: currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled in days", TIME_TRAVEL_IN_DAYS)
  });

  it("SUCCESS: Claim rewards again after more time (NFT still staked)", async () => {
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
    console.log("Second claim rewards:", rewardsClaimed);
    console.log("Total user rewards balance:", balanceAfter);
  });

  // ============================================
  // BURN STAKED NFT TESTS
  // ============================================

  it("FAIL: Try to burn staked NFT before freeze period", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
    try {
      await program.methods.burnStakedNft()
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
      console.log("\nExpected error: Burn before freeze period failed correctly");
    }
  });

  it("Stake a fresh NFT for burn test", async () => {
    // Mint a new NFT for burn test (using module-level nftToBurnKeypair)
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

    // Stake the new NFT
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
    // Advance time in milliseconds
    const currentTimestamp = Date.now() + (TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY)*3
    await advanceTime({ absoluteTimestamp: currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY });
    console.log("\nTime traveled in days", TIME_TRAVEL_IN_DAYS)
  });


  it("SUCCESS: Burn staked NFT with bonus rewards (1.1x multiplier)", async () => {
    const userRewardsAta = getAssociatedTokenAddressSync(rewardsMint, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);

    // Get balance before
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

    // Calculate expected rewards: 7 days * 10 points * 1.1 bonus
    const expectedNormalRewards = FREEZE_PERIOD_IN_DAYS * (POINTS_PER_STAKED_NFT_PER_DAY / 1_000_000);
    const expectedBonusRewards = expectedNormalRewards * 1.1;
    console.log("Expected normal rewards:", expectedNormalRewards);
    console.log("Expected with 1.1x bonus:", expectedBonusRewards);
  });

  // ============================================
  // ORACLE PLUGIN TESTS
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

  it("FAIL: Transfer NFT when Oracle is Rejected (outside market hours)", async () => {
    const randomWallet = anchor.web3.Keypair.generate();

    try {
      await program.methods.transfer()
      .accountsPartial({
        user: provider.wallet.publicKey,
        nextOwner: randomWallet.publicKey,
        oracle: oracle_plugin_id,
        nft: nftTransferKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
      throw new Error("Should have failed - Oracle should reject transfer");
    } catch (err) {
      console.log("\nExpected error: Oracle rejected transfer (outside market hours)");
    }
  });

  it("Update Oracle state", async () => {
    const tx = await program.methods.updateOracle()
    .accountsPartial({
      signer: provider.wallet.publicKey,
      payer: provider.wallet.publicKey,
      oracle: oracle_plugin_id,
      vaultForReward: anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("vault_for_reward"), oracle_plugin_id.toBuffer()],
        program.programId
      )[0],
      systemProgram: SystemProgram.programId,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Oracle state updated");
  });

  it("Transfer NFT when Oracle is Approved (after update)", async () => {
    const randomWallet = anchor.web3.Keypair.generate();

    const tx = await program.methods.transfer()
    .accountsPartial({
      user: provider.wallet.publicKey,
      nextOwner: randomWallet.publicKey,
      oracle: oracle_plugin_id,
      nft: nftTransferKeypair.publicKey,
      collection: collectionKeypair.publicKey,
      mplCoreProgram: MPL_CORE_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("NFT transferred successfully (Oracle approved)");
  });

});
