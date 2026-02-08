import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createInitializeMintInstruction,
  getMintLen,
  ExtensionType,
  createTransferCheckedWithTransferHookInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createInitializeTransferHookInstruction,
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
  createTransferCheckedInstruction,
} from "@solana/spl-token";
import { 
  SendTransactionError, 
  SystemProgram, 
  Transaction, 
  sendAndConfirmTransaction 
} from '@solana/web3.js';
import { WhitelistTransferHook } from "../target/types/whitelist_transfer_hook";

describe("whitelist-transfer-hook", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.whitelistTransferHook as Program<WhitelistTransferHook>;

  const mint2022 = anchor.web3.Keypair.generate();

  // Sender token account address
  const sourceTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    wallet.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  // Recipient token account address
  const recipient = anchor.web3.Keypair.generate();
  const destinationTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    recipient.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  // ExtraAccountMetaList address
  // Store extra accounts required by the custom transfer hook instruction
  const [extraAccountMetaListPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('extra-account-metas'), mint2022.publicKey.toBuffer()],
    program.programId,
  );
  
  const tokenAdmin = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("admin"),
      
    ],
    program.programId
  )[0];
  const whitelist = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("whitelist"),
      provider.publicKey.toBytes()
      
    ],
    program.programId
  )[0];
  
   const whitelist1 = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("whitelist"),
      destinationTokenAccount.toBytes()
      
    ],
    program.programId
  )[0];
  //all logs
  console.info("Token Admin:", tokenAdmin.toBase58());
  console.info("Whitelist:", whitelist.toBase58());
  console.info("Whitelist1:", whitelist1.toBase58());
  console.info("Mint2022:", mint2022.publicKey.toBase58());
  console.info("Source Token Account:", sourceTokenAccount.toBase58());
  console.info("Destination Token Account:", destinationTokenAccount.toBase58());
  console.info("Extra Account Meta List PDA:", extraAccountMetaListPDA.toBase58());

  console.info("==============================");



  it("Initializes the Whitelist", async () => {
    const tx = await program.methods.initializeWhitelist()
      .accountsPartial({
        admin: provider.publicKey,
        tokenAdmin,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("\nWhitelist initialized:", tokenAdmin.toBase58());
    console.log("Transaction signature:", tx);
  });

  it("Add user to whitelist", async () => {
    const tx = await program.methods.addToWhitelist(provider.publicKey)
      .accountsPartial({
        
        admin: provider.publicKey,
        tokenAdmin,
        whitelist,
      })
      .rpc();

    console.log("\nUser added to whitelist:", provider.publicKey.toBase58());
    console.log("Whitelist:", whitelist.toBase58());
    console.log("Transaction signature:", tx);
  });

  it("Remove user to whitelist", async () => {
    const tx = await program.methods.removeFromWhitelist(provider.publicKey)
      .accountsPartial({
        admin: provider.publicKey,
        whitelist,
      })
      .rpc();

    console.log("\nUser removed from whitelist:", provider.publicKey.toBase58());
    console.log("Transaction signature:", tx);
  });

  it('Create Mint Account with Transfer Hook Extension', async () => {
    const extensions = [ExtensionType.TransferHook];
    const mintLen = getMintLen(extensions);
    const lamports = await provider.connection.getMinimumBalanceForRentExemption(mintLen);

    const transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: mint2022.publicKey,
        space: mintLen,
        lamports: lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferHookInstruction(
        mint2022.publicKey,
        wallet.publicKey,
        program.programId, // Transfer Hook Program ID
        TOKEN_2022_PROGRAM_ID,
      ),
      createInitializeMintInstruction(mint2022.publicKey, 9, wallet.publicKey, null, TOKEN_2022_PROGRAM_ID),
    );

    const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet.payer, mint2022], {
      skipPreflight: true,
      commitment: 'confirmed',
    });

    const txDetails = await program.provider.connection.getTransaction(txSig, {
      maxSupportedTransactionVersion: 0,
      commitment: 'confirmed',
    });
    //console.log(txDetails.meta.logMessages);

    console.log("\nTransaction Signature: ", txSig);
  });
  xit("Create Mint with Transfer Hook Extension init _mint", async () => {
    try {
      
      const tx = await program.methods
        .initMint(9)
        .accountsPartial({
          user: wallet.publicKey,
          mint: mint2022.publicKey,
          extraAccountMetaList:extraAccountMetaListPDA,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([mint2022])
        .rpc();
      console.log("\nMint created:", mint2022.publicKey.toBase58());
      console.log("Transaction Signature:", tx);
    }
    catch (e) {
      console.error("This is the Error" , e);
    }
  
    });

  it('Create Token Accounts and Mint Tokens', async () => {
    // 100 tokens
    const amount = 100 * 10 ** 9;

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        sourceTokenAccount,
        wallet.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        destinationTokenAccount,
        recipient.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
      createMintToInstruction(mint2022.publicKey, sourceTokenAccount, wallet.publicKey, amount, [], TOKEN_2022_PROGRAM_ID),
    );

    const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet.payer], { skipPreflight: true });

    console.log("\nTransaction Signature: ", txSig);
  });

  // Account to store extra accounts required by the transfer hook instruction
  it('Create ExtraAccountMetaList Account', async () => {
    const initializeExtraAccountMetaListInstruction = await program.methods
      .initializeTransferHook()
      .accountsPartial({
        payer: wallet.publicKey,
        extraAccountMetaList: extraAccountMetaListPDA,
        mint: mint2022.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .instruction();
      //.rpc();

    const transaction = new Transaction().add(initializeExtraAccountMetaListInstruction);

    try {
      // Send the transaction

    const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet.payer], { skipPreflight: true, commitment: 'confirmed' });
      // console.log("\nTransfer Signature:", txSig);
    console.log("\nExtraAccountMetaList Account created:", extraAccountMetaListPDA.toBase58());
    console.log('Transaction Signature:', txSig);
    }
    catch (error) {
      if (error instanceof SendTransactionError) {
        // console.error("\nTransaction failed:", error.logs[6]);
        console.error("\nTransaction failed. Full logs:,");
         //error.getLogs(provider.connection)
         error.logs?.forEach((log, i) => console.error(`  ${i}: ${log}`));
      } else {
        console.error("\nUnexpected error:", error);
      }
    }
    }
  );

  it('Transfer Hook with Extra Account Meta', async () => {
    // 1 tokens
    const amount = 1 * 10 ** 9;
    const amountBigInt = BigInt(amount);

    // Create the base transfer instruction
    //  source: PublicKey,
 const transferInstruction = await createTransferCheckedWithTransferHookInstruction(
provider.connection,
         sourceTokenAccount,
         mint2022.publicKey,
         destinationTokenAccount,
         wallet.publicKey,
         amountBigInt,
         9,
         [],
'confirmed',
         TOKEN_2022_PROGRAM_ID,
       );
   
      
    //!or
    // const transferInstruction = await createTransferCheckedInstruction(

    //      sourceTokenAccount,
    //      mint2022.publicKey,
    //      destinationTokenAccount,
    //      wallet.publicKey,
    //      amountBigInt,
    //      9,
    //      [],

    //      TOKEN_2022_PROGRAM_ID,
    //    );
   
    //    // Manually add the extra accounts required by the transfer hook
    //    // These accounts are needed for the CPI to our transfer hook program
    //    transferInstruction.keys.push(
    //      // ExtraAccountMetaList PDA
    //      { pubkey: extraAccountMetaListPDA, isSigner: false, isWritable: false },

    //      {pubkey: whitelist, isSigner: false, isWritable: false},
    //     //  {pubkey: whitelist1, isSigner: false, isWritable: false},

    //       { pubkey: program.programId, isSigner: false, isWritable: false },




         
 
    //    );






    const transaction = new Transaction().add( transferInstruction);

    try {
      // Send the transaction
      const txSig = await sendAndConfirmTransaction(provider.connection, transaction, [wallet.payer], { skipPreflight: true, commitment: 'confirmed' });
      
      console.log('Transaction Signature:', txSig);
      
    }
    catch (error) {
      console.log(error);
      if (error instanceof SendTransactionError) {
        //console.error("\nTransaction failed:", error.logs[6]);
         console.error("\nTransaction failed. Full logs:", error.logs);
         error.logs?.forEach((log, i) => console.error(`  ${i}: ${log}`));
      } else {
        console.error("\nUnexpected error:", error);
      }
    }
  });
});
