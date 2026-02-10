#[cfg(test)]
mod tests {
    use {
        anchor_lang::{prelude::*, InstructionData, ToAccountMetas},
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_2022_PROGRAM_ID, CreateAssociatedTokenAccount,
        },
        solana_account::Account,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::path::PathBuf,
    };

    use crate::{
        constatnt::{EXTRA_META, USER, VAULT},
        state::{User, Vault},
    };

    static PROGRAM_ID: Pubkey = crate::ID;

    /// Setup function to initialize LiteSVM with the transfer hook vault program
    fn setup() -> (LiteSVM, Keypair) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        // Airdrop SOL to payer
        svm.airdrop(&payer.pubkey(), 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop");

        // Load the program
        let program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/transfer_hook_vault.so");

        let program_data = std::fs::read(&program_path).expect("Failed to read program file");

        svm.add_program(PROGRAM_ID, &program_data);

        msg!("✅ Setup complete - Program loaded: {}", PROGRAM_ID);
        (svm, payer)
    }

    #[test]
    fn test_create_vault_and_mint() {
        msg!("\n🧪 TEST: Create Vault and Mint\n");

        let (mut svm, payer) = setup();
        let admin = payer.pubkey();

        // Create mint keypair
        let mint = Keypair::new();
        msg!("Mint keypair: {}", mint.pubkey());

        // Derive vault PDA
        let (vault_pda, vault_bump) =
            Pubkey::find_program_address(&[VAULT.as_bytes(), admin.as_ref()], &PROGRAM_ID);
        msg!("Vault PDA: {} (bump: {})", vault_pda, vault_bump);

        // Create the instruction
        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::CreateVault {
                admin: admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::CrateVaultAndMint {
                fee: 50, // 0.5%
                name: "Vault Token".to_string(),
                symbol: "VTKN".to_string(),
                uri: "https://vault.io/token.json".to_string(),
                decimal: 9,
            }
            .data(),
        };

        // Send transaction
        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, blockhash);

        let result = svm.send_transaction(tx).unwrap();
        msg!("✅ Transaction successful");
        msg!("   Signature: {}", result.signature);
        msg!("   CUs consumed: {}", result.compute_units_consumed);

        // Verify vault account
        let vault_account = svm.get_account(&vault_pda).unwrap();
        let vault_data = Vault::try_deserialize(&mut vault_account.data.as_ref()).unwrap();

        assert_eq!(vault_data.admin, admin);
        assert_eq!(vault_data.mint_token, mint.pubkey());
        assert_eq!(vault_data.fees, 50);
        assert_eq!(vault_data.bump, vault_bump);

        msg!("✅ Vault verified:");
        msg!("   Admin: {}", vault_data.admin);
        msg!("   Mint: {}", vault_data.mint_token);
        msg!("   Fee: {}bps", vault_data.fees);
    }

    #[test]
    fn test_initialize_transfer_hook() {
        msg!("\n🧪 TEST: Initialize Transfer Hook\n");

        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();

        // First create vault
        let (vault_pda, _) =
            Pubkey::find_program_address(&[VAULT.as_bytes(), admin.as_ref()], &PROGRAM_ID);

        let create_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::CreateVault {
                admin: admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::CrateVaultAndMint {
                fee: 50,
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                uri: "https://test.com".to_string(),
                decimal: 9,
            }
            .data(),
        };

        let message = Message::new(&[create_vault_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, blockhash);
        svm.send_transaction(tx).unwrap();
        msg!("✅ Vault created");

        // Now initialize transfer hook
        let (extra_meta_pda, _) = Pubkey::find_program_address(
            &[EXTRA_META.as_bytes(), mint.pubkey().as_ref()],
            &PROGRAM_ID,
        );
        msg!("Extra meta PDA: {}", extra_meta_pda);

        let init_hook_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitializeExtraAccountMetaList {
                payer: admin,
                extra_account_meta_list: extra_meta_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::InitializeTransferHook {}.data(),
        };

        let message = Message::new(&[init_hook_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, blockhash);

        let result = svm.send_transaction(tx).unwrap();
        msg!("✅ Transfer hook initialized");
        msg!("   CUs consumed: {}", result.compute_units_consumed);

        // Verify extra account meta list exists
        let extra_meta_account = svm.get_account(&extra_meta_pda).unwrap();
        assert!(extra_meta_account.data.len() > 0);
        msg!(
            "✅ Extra account meta list verified (size: {} bytes)",
            extra_meta_account.data.len()
        );
    }

    #[test]
    fn test_whitelist_operations() {
        msg!("\n🧪 TEST: Whitelist Operations\n");

        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();

        let user1 = Keypair::new();
        svm.airdrop(&user1.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        // Create vault first
        let (vault_pda, _) =
            Pubkey::find_program_address(&[VAULT.as_bytes(), admin.as_ref()], &PROGRAM_ID);

        let create_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::CreateVault {
                admin: admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::CrateVaultAndMint {
                fee: 50,
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                uri: "https://test.com".to_string(),
                decimal: 9,
            }
            .data(),
        };

        let message = Message::new(&[create_vault_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, blockhash);
        svm.send_transaction(tx).unwrap();
        msg!("✅ Vault created");

        // Add user to whitelist
        let (user_pda, user_bump) =
            Pubkey::find_program_address(&[USER.as_bytes(), user1.pubkey().as_ref()], &PROGRAM_ID);
        msg!("User PDA: {} (bump: {})", user_pda, user_bump);

        let add_whitelist_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::WhitelistOperations {
                admin: admin,
                vault: vault_pda,
                user: user_pda,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::AddToWhitelist {
                user: user1.pubkey(),
            }
            .data(),
        };

        let message = Message::new(&[add_whitelist_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, blockhash);

        let result = svm.send_transaction(tx).unwrap();
        msg!("✅ User added to whitelist");
        msg!("   CUs consumed: {}", result.compute_units_consumed);

        // Verify user account
        let user_account = svm.get_account(&user_pda).unwrap();
        let user_data = User::try_deserialize(&mut user_account.data.as_ref()).unwrap();

        assert_eq!(user_data.address, user1.pubkey());
        assert_eq!(user_data.bump, user_bump);
        msg!("✅ User whitelist verified:");
        msg!("   Address: {}", user_data.address);

        // Test remove from whitelist
        let remove_whitelist_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::WhitelistOperations {
                admin: admin,
                vault: vault_pda,
                user: user_pda,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::RemoveFromWhitelist {
                user: user1.pubkey(),
            }
            .data(),
        };

        let message = Message::new(&[remove_whitelist_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, blockhash);

        svm.send_transaction(tx).unwrap();
        msg!("✅ User removed from whitelist");

        // Verify account is closed
        let user_account_after = svm.get_account(&user_pda);
        assert!(user_account_after.is_none());
        msg!("✅ User account closed successfully");
    }

    #[test]
    fn test_deposit_and_withdraw() {
        msg!("\n🧪 TEST: Deposit and Withdraw\n");

        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();

        let user1 = Keypair::new();
        svm.airdrop(&user1.pubkey(), 100 * LAMPORTS_PER_SOL)
            .unwrap();

        // Create vault
        let (vault_pda, _) =
            Pubkey::find_program_address(&[VAULT.as_bytes(), admin.as_ref()], &PROGRAM_ID);

        let create_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::CreateVault {
                admin: admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::CrateVaultAndMint {
                fee: 50,
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                uri: "https://test.com".to_string(),
                decimal: 9,
            }
            .data(),
        };

        let message = Message::new(&[create_vault_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, blockhash);
        svm.send_transaction(tx).unwrap();
        msg!("✅ Vault created");

        // Add user to whitelist
        let (user_pda, _) =
            Pubkey::find_program_address(&[USER.as_bytes(), user1.pubkey().as_ref()], &PROGRAM_ID);

        let add_whitelist_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::WhitelistOperations {
                admin: admin,
                vault: vault_pda,
                user: user_pda,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::AddToWhitelist {
                user: user1.pubkey(),
            }
            .data(),
        };

        let message = Message::new(&[add_whitelist_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, blockhash);
        svm.send_transaction(tx).unwrap();
        msg!("✅ User whitelisted");

        // Create user's ATA
        let user_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint.pubkey())
            .owner(&user1.pubkey())
            .token_program_id(&TOKEN_2022_PROGRAM_ID)
            .send()
            .unwrap();
        msg!("✅ User ATA created: {}", user_ata);

        // Test Deposit
        let deposit_amount = 5 * LAMPORTS_PER_SOL;
        let vault_balance_before = svm.get_balance(&vault_pda).unwrap_or(0);

        let deposit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Deposit {
                owner: user1.pubkey(),
                mint: mint.pubkey(),
                vault: vault_pda,
                user: user_pda,
                owner_ata: user_ata,
                associated_token_program: anchor_spl::associated_token::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Deposit {
                amount: deposit_amount,
            }
            .data(),
        };

        let message = Message::new(&[deposit_ix], Some(&user1.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&user1], message, blockhash);

        let result = svm.send_transaction(tx).unwrap();
        msg!("✅ Deposit successful");
        msg!("   Amount: {} SOL", deposit_amount / LAMPORTS_PER_SOL);
        msg!("   CUs consumed: {}", result.compute_units_consumed);

        // Verify deposit
        let vault_balance_after = svm.get_balance(&vault_pda).unwrap_or(0);
        assert_eq!(vault_balance_after - vault_balance_before, deposit_amount);
        msg!(
            "✅ Vault balance increased by {} SOL",
            deposit_amount / LAMPORTS_PER_SOL
        );

        // Test Withdraw
        let withdraw_amount = 2 * LAMPORTS_PER_SOL;
        let vault_balance_before_withdraw = svm.get_balance(&vault_pda).unwrap_or(0);

        let withdraw_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Withdraw {
                owner: user1.pubkey(),
                mint: mint.pubkey(),
                vault: vault_pda,
                user: user_pda,
                owner_ata: user_ata,
                associated_token_program: anchor_spl::associated_token::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Withdraw {
                amount: withdraw_amount,
            }
            .data(),
        };

        let message = Message::new(&[withdraw_ix], Some(&user1.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&user1], message, blockhash);

        let result = svm.send_transaction(tx).unwrap();
        msg!("✅ Withdraw successful");
        msg!("   Amount: {} SOL", withdraw_amount / LAMPORTS_PER_SOL);
        msg!("   CUs consumed: {}", result.compute_units_consumed);

        // Verify withdraw
        let vault_balance_after_withdraw = svm.get_balance(&vault_pda).unwrap_or(0);
        assert_eq!(
            vault_balance_before_withdraw - vault_balance_after_withdraw,
            withdraw_amount
        );
        msg!(
            "✅ Vault balance decreased by {} SOL",
            withdraw_amount / LAMPORTS_PER_SOL
        );
    }

    #[test]
    fn test_non_admin_cannot_whitelist() {
        msg!("\n🧪 TEST: Non-Admin Cannot Whitelist\n");

        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();

        let attacker = Keypair::new();
        svm.airdrop(&attacker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();

        let target_user = Keypair::new();

        // Create vault
        let (vault_pda, _) =
            Pubkey::find_program_address(&[VAULT.as_bytes(), admin.as_ref()], &PROGRAM_ID);

        let create_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::CreateVault {
                admin: admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::CrateVaultAndMint {
                fee: 50,
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                uri: "https://test.com".to_string(),
                decimal: 9,
            }
            .data(),
        };

        let message = Message::new(&[create_vault_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, blockhash);
        svm.send_transaction(tx).unwrap();

        // Try to whitelist as non-admin
        let (user_pda, _) = Pubkey::find_program_address(
            &[USER.as_bytes(), target_user.pubkey().as_ref()],
            &PROGRAM_ID,
        );

        let add_whitelist_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::WhitelistOperations {
                admin: attacker.pubkey(), // Wrong admin!
                vault: vault_pda,
                user: user_pda,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::AddToWhitelist {
                user: target_user.pubkey(),
            }
            .data(),
        };

        let message = Message::new(&[add_whitelist_ix], Some(&attacker.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&attacker], message, blockhash);

        let result = svm.send_transaction(tx);
        assert!(result.is_err(), "Non-admin should not be able to whitelist");
        msg!("✅ Non-admin correctly rejected from whitelisting");
    }

    #[test]
    fn test_full_workflow() {
        msg!("\n🧪 TEST: Full Workflow (Create → Whitelist → Deposit → Withdraw)\n");

        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();

        let user1 = Keypair::new();
        let user2 = Keypair::new();
        svm.airdrop(&user1.pubkey(), 50 * LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&user2.pubkey(), 50 * LAMPORTS_PER_SOL).unwrap();

        // 1. Create vault
        msg!("\n1️⃣ Creating vault...");
        let (vault_pda, _) =
            Pubkey::find_program_address(&[VAULT.as_bytes(), admin.as_ref()], &PROGRAM_ID);

        let create_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::CreateVault {
                admin: admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::CrateVaultAndMint {
                fee: 50,
                name: "Vault Token".to_string(),
                symbol: "VTKN".to_string(),
                uri: "https://vault.io/token.json".to_string(),
                decimal: 9,
            }
            .data(),
        };

        let message = Message::new(&[create_vault_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, blockhash);
        svm.send_transaction(tx).unwrap();
        msg!("   ✅ Vault created");

        // 2. Initialize transfer hook
        msg!("\n2️⃣ Initializing transfer hook...");
        let (extra_meta_pda, _) = Pubkey::find_program_address(
            &[EXTRA_META.as_bytes(), mint.pubkey().as_ref()],
            &PROGRAM_ID,
        );

        let init_hook_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitializeExtraAccountMetaList {
                payer: admin,
                extra_account_meta_list: extra_meta_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::InitializeTransferHook {}.data(),
        };

        let message = Message::new(&[init_hook_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, blockhash);
        svm.send_transaction(tx).unwrap();
        msg!("   ✅ Transfer hook initialized");

        // 3. Whitelist users
        msg!("\n3️⃣ Whitelisting users...");
        let (user1_pda, _) =
            Pubkey::find_program_address(&[USER.as_bytes(), user1.pubkey().as_ref()], &PROGRAM_ID);
        let (user2_pda, _) =
            Pubkey::find_program_address(&[USER.as_bytes(), user2.pubkey().as_ref()], &PROGRAM_ID);

        for (user_pubkey, user_pda) in [(user1.pubkey(), user1_pda), (user2.pubkey(), user2_pda)] {
            let add_whitelist_ix = Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::WhitelistOperations {
                    admin: admin,
                    vault: vault_pda,
                    user: user_pda,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::AddToWhitelist { user: user_pubkey }.data(),
            };

            let message = Message::new(&[add_whitelist_ix], Some(&payer.pubkey()));
            let blockhash = svm.latest_blockhash();
            let tx = Transaction::new(&[&payer], message, blockhash);
            svm.send_transaction(tx).unwrap();
            msg!("   ✅ User {} whitelisted", user_pubkey);
        }

        // 4. Create ATAs
        msg!("\n4️⃣ Creating token accounts...");
        let user1_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint.pubkey())
            .owner(&user1.pubkey())
            .token_program_id(&TOKEN_2022_PROGRAM_ID)
            .send()
            .unwrap();
        let user2_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint.pubkey())
            .owner(&user2.pubkey())
            .token_program_id(&TOKEN_2022_PROGRAM_ID)
            .send()
            .unwrap();
        msg!("   ✅ ATAs created");

        // 5. User1 deposits
        msg!("\n5️⃣ User1 depositing...");
        let deposit_amount = 10 * LAMPORTS_PER_SOL;
        let deposit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Deposit {
                owner: user1.pubkey(),
                mint: mint.pubkey(),
                vault: vault_pda,
                user: user1_pda,
                owner_ata: user1_ata,
                associated_token_program: anchor_spl::associated_token::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Deposit {
                amount: deposit_amount,
            }
            .data(),
        };

        let message = Message::new(&[deposit_ix], Some(&user1.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&user1], message, blockhash);
        svm.send_transaction(tx).unwrap();
        msg!(
            "   ✅ User1 deposited {} SOL",
            deposit_amount / LAMPORTS_PER_SOL
        );

        // 6. User1 withdraws
        msg!("\n6️⃣ User1 withdrawing...");
        let withdraw_amount = 3 * LAMPORTS_PER_SOL;
        let withdraw_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Withdraw {
                owner: user1.pubkey(),
                mint: mint.pubkey(),
                vault: vault_pda,
                user: user1_pda,
                owner_ata: user1_ata,
                associated_token_program: anchor_spl::associated_token::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Withdraw {
                amount: withdraw_amount,
            }
            .data(),
        };

        let message = Message::new(&[withdraw_ix], Some(&user1.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&user1], message, blockhash);
        svm.send_transaction(tx).unwrap();
        msg!(
            "   ✅ User1 withdrew {} SOL",
            withdraw_amount / LAMPORTS_PER_SOL
        );

        // 7. Verify final state
        msg!("\n7️⃣ Verifying final state...");
        let vault_balance = svm.get_balance(&vault_pda).unwrap_or(0);
        let expected_vault_balance = deposit_amount - withdraw_amount;
        assert_eq!(vault_balance, expected_vault_balance);
        msg!(
            "   ✅ Vault balance: {} SOL",
            vault_balance / LAMPORTS_PER_SOL
        );

        msg!("\n✅ Full workflow completed successfully!");
    }

    #[test]
    #[should_panic]
    fn test_cannot_deposit_without_whitelist() {
        msg!("\n🧪 TEST: Cannot Deposit Without Whitelist\n");

        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();

        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        // Create vault
        let (vault_pda, _) =
            Pubkey::find_program_address(&[VAULT.as_bytes(), admin.as_ref()], &PROGRAM_ID);

        let create_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::CreateVault {
                admin: admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::CrateVaultAndMint {
                fee: 50,
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                uri: "https://test.com".to_string(),
                decimal: 9,
            }
            .data(),
        };

        let message = Message::new(&[create_vault_ix], Some(&payer.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, blockhash);
        svm.send_transaction(tx).unwrap();

        // Try to deposit without whitelist (should fail)
        let (user_pda, _) =
            Pubkey::find_program_address(&[USER.as_bytes(), user.pubkey().as_ref()], &PROGRAM_ID);

        let user_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint.pubkey())
            .owner(&user.pubkey())
            .token_program_id(&TOKEN_2022_PROGRAM_ID)
            .send()
            .unwrap();

        let deposit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Deposit {
                owner: user.pubkey(),
                mint: mint.pubkey(),
                vault: vault_pda,
                user: user_pda,
                owner_ata: user_ata,
                associated_token_program: anchor_spl::associated_token::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Deposit {
                amount: 1 * LAMPORTS_PER_SOL,
            }
            .data(),
        };

        let message = Message::new(&[deposit_ix], Some(&user.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&user], message, blockhash);

        // This should panic because user is not whitelisted
        svm.send_transaction(tx).unwrap();
    }
}
