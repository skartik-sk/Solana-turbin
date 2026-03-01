#[cfg(test)]
mod tests {
    use litesvm::LiteSVM;
    use litesvm_token::{spl_token, CreateAssociatedTokenAccount, CreateMint, MintTo};
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::get_associated_token_address;
    use std::{path::PathBuf, sync::Mutex};

    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID_STR: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    static CU_RESULTS: Mutex<Vec<(&'static str, u64)>> = Mutex::new(Vec::new());

    fn record_cu(name: &'static str, cu: u64) {
        CU_RESULTS.lock().unwrap().push((name, cu));
    }

    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn system_program_id() -> Pubkey {
        solana_sdk_ids::system_program::ID
    }

    fn associated_token_program_id() -> Pubkey {
        ASSOCIATED_TOKEN_PROGRAM_ID_STR.parse().unwrap()
    }

    /// Load program, create payer (maker) and contributor keypairs, airdrop SOL.
    fn setup() -> (LiteSVM, Keypair, Keypair) {
        let mut svm = LiteSVM::new();
        let maker = Keypair::new();
        let contributor = Keypair::new();

        svm.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop to maker failed");
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop to contributor failed");

        // Load the compiled program .so file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/sbpf-solana-solana/release/fundraiser.so");
        let program_data = std::fs::read(&so_path)
            .unwrap_or_else(|_| panic!("Failed to read program SO file at {:?}", so_path));
        svm.add_program(program_id(), &program_data)
            .expect("Failed to load program");

        (svm, maker, contributor)
    }

    /// Derive the fundraiser PDA from the maker's public key.
    fn fundraiser_pda(maker: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"fundraiser", maker.as_ref()],
            &program_id(),
        )
    }

    /// Derive the contributor account PDA.
    fn contributor_pda(fundraiser: &Pubkey, contributor: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"contributor", fundraiser.as_ref(), contributor.as_ref()],
            &program_id(),
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Helper: send a transaction and unwrap, returning compute units consumed.
    // ─────────────────────────────────────────────────────────────────────────
    fn send_tx(
        svm: &mut LiteSVM,
        instructions: &[Instruction],
        payer: &Pubkey,
        signers: &[&Keypair],
    ) -> u64 {
        let message = Message::new(instructions, Some(payer));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(signers, message, recent_blockhash);
        let result = svm.send_transaction(tx).expect("Transaction failed");
        result.compute_units_consumed
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1 (mirrors TS "Test Preparation"):
    //   - Airdrop, create mint with 6 decimals, create ATAs, mint 10 tokens
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_initialize_fundraiser() {
        let (mut svm, maker, contributor) = setup();

        // Create mint with 6 decimals, authority = contributor (matches TS: payer mints)
        let mint = CreateMint::new(&mut svm, &contributor)
            .decimals(6)
            .authority(&contributor.pubkey())
            .send()
            .unwrap();
        println!("Mint: {}", mint);

        // Create contributor ATA (they hold the tokens)
        let contributor_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&contributor.pubkey())
            .send()
            .unwrap();

        // Create maker ATA (receives tokens on successful claim)
        let maker_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        // Mint 10 tokens (10 * 10^6 = 10_000_000 raw units) to contributor
        MintTo::new(&mut svm, &contributor, &mint, &contributor_ata, 10_000_000)
            .send()
            .unwrap();
        println!("Minted 10 tokens to contributor ATA");

        // ── Derive PDAs ──────────────────────────────────────────────────────
        let (fundraiser, fundraiser_bump) = fundraiser_pda(&maker.pubkey());
        let vault = get_associated_token_address(&fundraiser, &mint);

        println!("Fundraiser PDA: {}, bump: {}", fundraiser, fundraiser_bump);
        println!("Vault ATA:      {}", vault);

        // ── Build Initialize instruction ─────────────────────────────────────
        // data (handler receives after discriminator stripped):
        //   [0]     bump:     u8
        //   [1..9]  amount:   u64 LE   → 30_000_000 (matches TS: new BN(30000000))
        //   [9]     duration: u8       → 0           (matches TS: 0)
        let amount_to_raise: u64 = 30_000_000;
        let duration: u8 = 0;
        let init_data: Vec<u8> = [
            vec![0u8],                              // discriminator
            vec![fundraiser_bump],                  // bump
            amount_to_raise.to_le_bytes().to_vec(), // amount
            vec![duration],                         // duration
        ]
        .concat();

        let init_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: init_data,
        };

        let cu = send_tx(&mut svm, &[init_ix], &maker.pubkey(), &[&maker]);
        println!("Initialize OK — CUs: {}", cu);
        record_cu("initialize/base", cu);

        // Verify fundraiser account data
        let fund_account = svm.get_account(&fundraiser).unwrap();
        let fund_data = fund_account.data;
        assert_eq!(fund_data.len(), 90, "Fundraiser account should be 90 bytes");
        let stored_maker: [u8; 32] = fund_data[0..32].try_into().unwrap();
        assert_eq!(stored_maker, maker.pubkey().to_bytes(), "maker mismatch");
        let stored_amount = u64::from_le_bytes(fund_data[64..72].try_into().unwrap());
        assert_eq!(stored_amount, amount_to_raise, "amount_to_raise mismatch");
        let stored_duration = fund_data[88];
        assert_eq!(stored_duration, duration, "duration mismatch");
        let stored_bump = fund_data[89];
        assert_eq!(stored_bump, fundraiser_bump, "bump mismatch");
        println!("Fundraiser account data verified ✓");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2 (mirrors TS "Contribute to Fundraiser" x2):
    //   - Initialize then contribute twice with 1_000_000
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_contribute() {
        let (mut svm, maker, contributor) = setup();

        let mint = CreateMint::new(&mut svm, &contributor)
            .decimals(6)
            .authority(&contributor.pubkey())
            .send()
            .unwrap();

        let contributor_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&contributor.pubkey())
            .send()
            .unwrap();

        CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        // Mint 10 tokens to contributor
        MintTo::new(&mut svm, &contributor, &mint, &contributor_ata, 10_000_000)
            .send()
            .unwrap();

        // ── Initialize ───────────────────────────────────────────────────────
        let (fundraiser, fundraiser_bump) = fundraiser_pda(&maker.pubkey());
        let vault = get_associated_token_address(&fundraiser, &mint);

        let amount_to_raise: u64 = 30_000_000;
        let init_data: Vec<u8> = [
            vec![0u8],
            vec![fundraiser_bump],
            amount_to_raise.to_le_bytes().to_vec(),
            vec![0u8], // duration = 0
        ]
        .concat();
        let init_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: init_data,
        };
        send_tx(&mut svm, &[init_ix], &maker.pubkey(), &[&maker]);
        println!("Initialized fundraiser ✓");

        // ── Contribute #1 — 1_000_000 ────────────────────────────────────────
        let (contributor_account, contributor_bump) =
            contributor_pda(&fundraiser, &contributor.pubkey());

        let contribute_amount: u64 = 1_000_000;
        let contribute_data: Vec<u8> = [
            vec![1u8],                               // discriminator
            contribute_amount.to_le_bytes().to_vec(), // amount
            vec![contributor_bump],                  // contributor_bump
        ]
        .concat();
        let contribute_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(contributor_account, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data: contribute_data.clone(),
        };
        let cu1 = send_tx(
            &mut svm,
            &[contribute_ix],
            &contributor.pubkey(),
            &[&contributor],
        );
        println!("Contribute #1 OK — CUs: {}", cu1);
        record_cu("contribute/base", cu1);

        // Verify vault balance = 1_000_000
        let vault_balance = get_token_balance(&svm, &vault);
        assert_eq!(vault_balance, 1_000_000, "Vault should have 1_000_000 after first contribution");
        println!("Vault balance after contribute #1: {}", vault_balance);

        // Verify contributor account amount = 1_000_000
        let ca_amount = get_contributor_account_amount(&svm, &contributor_account);
        assert_eq!(ca_amount, 1_000_000);
        println!("Contributor account amount: {}", ca_amount);

        // ── Contribute #2 — 1_000_000 ────────────────────────────────────────
        let contribute_ix2 = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(contributor_account, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data: contribute_data,
        };
        let cu2 = send_tx(
            &mut svm,
            &[contribute_ix2],
            &contributor.pubkey(),
            &[&contributor],
        );
        println!("Contribute #2 OK — CUs: {}", cu2);

        let vault_balance2 = get_token_balance(&svm, &vault);
        assert_eq!(vault_balance2, 2_000_000, "Vault should have 2_000_000 after second contribution");
        println!("Vault balance after contribute #2: {}", vault_balance2);

        let ca_amount2 = get_contributor_account_amount(&svm, &contributor_account);
        assert_eq!(ca_amount2, 2_000_000);
        println!("Contributor account amount: {}", ca_amount2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 3 (mirrors TS "Contribute to Fundraiser - Robustness Test"):
    //   Attempt to contribute 2_000_000 after already having 2_000_000 →
    //   total would be 4_000_000 > 10% of 30_000_000 (= 3_000_000) → should FAIL
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_contribute_robustness() {
        let (mut svm, maker, contributor) = setup();

        let mint = CreateMint::new(&mut svm, &contributor)
            .decimals(6)
            .authority(&contributor.pubkey())
            .send()
            .unwrap();

        let contributor_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&contributor.pubkey())
            .send()
            .unwrap();

        CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        MintTo::new(&mut svm, &contributor, &mint, &contributor_ata, 10_000_000)
            .send()
            .unwrap();

        let (fundraiser, fundraiser_bump) = fundraiser_pda(&maker.pubkey());
        let vault = get_associated_token_address(&fundraiser, &mint);

        let amount_to_raise: u64 = 30_000_000;
        let init_data: Vec<u8> = [
            vec![0u8],
            vec![fundraiser_bump],
            amount_to_raise.to_le_bytes().to_vec(),
            vec![0u8],
        ]
        .concat();
        let init_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: init_data,
        };
        send_tx(&mut svm, &[init_ix], &maker.pubkey(), &[&maker]);

        let (contributor_account, contributor_bump) =
            contributor_pda(&fundraiser, &contributor.pubkey());

        // Two successful contributions of 1_000_000 each
        for _ in 0..2 {
            let data: Vec<u8> = [
                vec![1u8],
                1_000_000u64.to_le_bytes().to_vec(),
                vec![contributor_bump],
            ]
            .concat();
            let ix = Instruction {
                program_id: program_id(),
                accounts: vec![
                    AccountMeta::new(contributor.pubkey(), true),
                    AccountMeta::new_readonly(mint, false),
                    AccountMeta::new(fundraiser, false),
                    AccountMeta::new(contributor_account, false),
                    AccountMeta::new(contributor_ata, false),
                    AccountMeta::new(vault, false),
                    AccountMeta::new_readonly(system_program_id(), false),
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                ],
                data,
            };
            send_tx(&mut svm, &[ix], &contributor.pubkey(), &[&contributor]);
        }

        // Third contribution of 2_000_000: total = 4_000_000 > 3_000_000 (10%) → must fail
        let bad_data: Vec<u8> = [
            vec![1u8],
            2_000_000u64.to_le_bytes().to_vec(),
            vec![contributor_bump],
        ]
        .concat();
        let bad_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(contributor_account, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data: bad_data,
        };
        let message = Message::new(&[bad_ix], Some(&contributor.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&contributor], message, blockhash);
        let result = svm.send_transaction(tx);
        assert!(result.is_err(), "Expected MaximumContributionsReached error but transaction succeeded");
        println!("Robustness test passed — over-contribution correctly rejected ✓");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 4 (mirrors TS "Check contributions - Robustness Test"):
    //   Try to claim while vault (2_000_000) < target (30_000_000) → must FAIL
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_check_contributions_robustness() {
        let (mut svm, maker, contributor) = setup();

        let mint = CreateMint::new(&mut svm, &contributor)
            .decimals(6)
            .authority(&contributor.pubkey())
            .send()
            .unwrap();

        let contributor_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&contributor.pubkey())
            .send()
            .unwrap();

        let maker_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        MintTo::new(&mut svm, &contributor, &mint, &contributor_ata, 10_000_000)
            .send()
            .unwrap();

        let (fundraiser, fundraiser_bump) = fundraiser_pda(&maker.pubkey());
        let vault = get_associated_token_address(&fundraiser, &mint);

        let init_data: Vec<u8> = [
            vec![0u8],
            vec![fundraiser_bump],
            30_000_000u64.to_le_bytes().to_vec(),
            vec![0u8],
        ]
        .concat();
        let init_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: init_data,
        };
        send_tx(&mut svm, &[init_ix], &maker.pubkey(), &[&maker]);

        let (contributor_account, contributor_bump) =
            contributor_pda(&fundraiser, &contributor.pubkey());

        let contrib_data: Vec<u8> = [
            vec![1u8],
            1_000_000u64.to_le_bytes().to_vec(),
            vec![contributor_bump],
        ]
        .concat();
        let contrib_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(contributor_account, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            ],
            data: contrib_data,
        };
        send_tx(&mut svm, &[contrib_ix], &contributor.pubkey(), &[&contributor]);

        // Try to check contributions (should fail: target not met)
        let check_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(maker_ata, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: vec![2u8], // discriminator only
        };
        let message = Message::new(&[check_ix], Some(&maker.pubkey()));
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&maker], message, blockhash);
        let result = svm.send_transaction(tx);
        assert!(result.is_err(), "Expected TargetNotMet error but transaction succeeded");
        println!("Check contributions robustness test passed — target-not-met correctly rejected ✓");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 5 (mirrors TS "Refund Contributions"):
    //   Initialize, contribute 2x, then refund — should succeed since
    //   duration=0 means the fundraiser is immediately "ended" (elapsed >= 0)
    //   and the target was not met.
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_refund() {
        let (mut svm, maker, contributor) = setup();

        let mint = CreateMint::new(&mut svm, &contributor)
            .decimals(6)
            .authority(&contributor.pubkey())
            .send()
            .unwrap();

        let contributor_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&contributor.pubkey())
            .send()
            .unwrap();

        CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        MintTo::new(&mut svm, &contributor, &mint, &contributor_ata, 10_000_000)
            .send()
            .unwrap();

        let (fundraiser, fundraiser_bump) = fundraiser_pda(&maker.pubkey());
        let vault = get_associated_token_address(&fundraiser, &mint);

        // Initialize with duration=0 (immediately ended, but target not met)
        let init_data: Vec<u8> = [
            vec![0u8],
            vec![fundraiser_bump],
            30_000_000u64.to_le_bytes().to_vec(),
            vec![0u8], // duration = 0
        ]
        .concat();
        let init_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: init_data,
        };
        send_tx(&mut svm, &[init_ix], &maker.pubkey(), &[&maker]);

        let (contributor_account, contributor_bump) =
            contributor_pda(&fundraiser, &contributor.pubkey());

        // Contribute twice (1_000_000 each)
        for _ in 0..2 {
            let data: Vec<u8> = [
                vec![1u8],
                1_000_000u64.to_le_bytes().to_vec(),
                vec![contributor_bump],
            ]
            .concat();
            let ix = Instruction {
                program_id: program_id(),
                accounts: vec![
                    AccountMeta::new(contributor.pubkey(), true),
                    AccountMeta::new_readonly(mint, false),
                    AccountMeta::new(fundraiser, false),
                    AccountMeta::new(contributor_account, false),
                    AccountMeta::new(contributor_ata, false),
                    AccountMeta::new(vault, false),
                    AccountMeta::new_readonly(system_program_id(), false),
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                ],
                data,
            };
            send_tx(&mut svm, &[ix], &contributor.pubkey(), &[&contributor]);
        }

        let vault_balance_before = get_token_balance(&svm, &vault);
        println!("Vault balance before refund: {}", vault_balance_before);
        assert_eq!(vault_balance_before, 2_000_000);

        let ca_amount = get_contributor_account_amount(&svm, &contributor_account);
        println!("Contributor account amount before refund: {}", ca_amount);
        assert_eq!(ca_amount, 2_000_000);

        // ── Refund ───────────────────────────────────────────────────────────
        let refund_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new_readonly(maker.pubkey(), false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(contributor_account, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: vec![3u8], // discriminator only
        };
        let cu = send_tx(
            &mut svm,
            &[refund_ix],
            &contributor.pubkey(),
            &[&contributor],
        );
        println!("Refund OK — CUs: {}", cu);
        record_cu("refund/base", cu);

        let vault_balance_after = get_token_balance(&svm, &vault);
        println!("Vault balance after refund: {}", vault_balance_after);
        assert_eq!(vault_balance_after, 0, "Vault should be empty after refund");

        let contributor_ata_balance = get_token_balance(&svm, &contributor_ata);
        println!("Contributor ATA balance after refund: {}", contributor_ata_balance);
        assert_eq!(contributor_ata_balance, 10_000_000, "Contributor should have all tokens back");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 6: Full happy-path — initialize, fill target, claim
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_check_contributions_success() {
        let (mut svm, maker, contributor) = setup();

        let mint = CreateMint::new(&mut svm, &contributor)
            .decimals(6)
            .authority(&contributor.pubkey())
            .send()
            .unwrap();

        let contributor_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&contributor.pubkey())
            .send()
            .unwrap();

        let maker_ata = CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        // Mint exactly the target amount to contributor
        let amount_to_raise: u64 = 3_000_000; // small enough to contribute in one tx (within 10% rule of 30M)
        // Use a smaller raise target so 10% = 300_000 * 10 = 3_000_000 fits within rules
        // Actually: target=3_000_000, max_contribution=300_000 per tx.
        // With 10 contributions of 300_000 we'd reach the target. That's too many txs.
        // Instead: target=1_000_000, max_contribution=100_000 → need 10 contributions.
        // Simpler: target=500_000, max=50_000 → 10 contributions.
        // SIMPLEST for test: use 1 contributor that contributes the full amount in one go:
        // target = amount * 10 so that 10% = amount.
        // E.g. target = 10_000_000, contributor has 10_000_000. Max contribution = 1_000_000.
        // Need 10 contributions to reach target. Too many. Let's just mint the full target
        // and use target = 1_000_000, contributes 100_000 x 10 times.
        // For test simplicity, let's just do a single contribution that reaches target:
        // target = 1_000_000, max_contribution = 100_000 → can't reach in one go.
        // This is a fundamental design constraint: 10% rule means you need at least 10 contributions.
        // Let's mint enough and do 10 contributions:
        let target: u64 = 1_000_000;
        let per_contribution: u64 = target / 10; // = 100_000 (exactly 10%)
        MintTo::new(&mut svm, &contributor, &mint, &contributor_ata, target + per_contribution)
            .send()
            .unwrap();

        let (fundraiser, fundraiser_bump) = fundraiser_pda(&maker.pubkey());
        let vault = get_associated_token_address(&fundraiser, &mint);

        // Initialize with small target that can actually be filled
        let init_data: Vec<u8> = [
            vec![0u8],
            vec![fundraiser_bump],
            target.to_le_bytes().to_vec(),
            vec![0u8],
        ]
        .concat();
        let init_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: init_data,
        };
        send_tx(&mut svm, &[init_ix], &maker.pubkey(), &[&maker]);

        let (contributor_account, contributor_bump) =
            contributor_pda(&fundraiser, &contributor.pubkey());

        // Contribute 10 times to reach the target
        for i in 0..10 {
            let data: Vec<u8> = [
                vec![1u8],
                per_contribution.to_le_bytes().to_vec(),
                vec![contributor_bump],
            ]
            .concat();
            let ix = Instruction {
                program_id: program_id(),
                accounts: vec![
                    AccountMeta::new(contributor.pubkey(), true),
                    AccountMeta::new_readonly(mint, false),
                    AccountMeta::new(fundraiser, false),
                    AccountMeta::new(contributor_account, false),
                    AccountMeta::new(contributor_ata, false),
                    AccountMeta::new(vault, false),
                    AccountMeta::new_readonly(system_program_id(), false),
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                ],
                data,
            };
            send_tx(&mut svm, &[ix], &contributor.pubkey(), &[&contributor]);
            println!("Contribution {} done", i + 1);
        }

        let vault_balance = get_token_balance(&svm, &vault);
        println!("Vault balance before claim: {}", vault_balance);
        assert_eq!(vault_balance, target);

        // ── Check Contributions (claim) ──────────────────────────────────────
        let check_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(maker_ata, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
            ],
            data: vec![2u8],
        };
        let cu = send_tx(&mut svm, &[check_ix], &maker.pubkey(), &[&maker]);
        println!("Check contributions OK — CUs: {}", cu);
        record_cu("check_contributions/base", cu);

        let maker_ata_balance = get_token_balance(&svm, &maker_ata);
        println!("Maker ATA balance after claim: {}", maker_ata_balance);
        assert_eq!(maker_ata_balance, target, "Maker should receive all raised tokens");

        let vault_balance_after = get_token_balance(&svm, &vault);
        assert_eq!(vault_balance_after, 0, "Vault should be empty after claim");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Utilities
    // ─────────────────────────────────────────────────────────────────────────

    /// Read the token balance (amount) from a token account.
    fn get_token_balance(svm: &LiteSVM, token_account: &Pubkey) -> u64 {
        let account = svm.get_account(token_account)
            .expect("Token account not found");
        // SPL token account layout: amount is at offset 64..72
        u64::from_le_bytes(account.data[64..72].try_into().unwrap())
    }

    /// Read the `amount` field from a Contributor PDA account (first 8 bytes).
    fn get_contributor_account_amount(svm: &LiteSVM, contributor_account: &Pubkey) -> u64 {
        let account = svm.get_account(contributor_account)
            .expect("Contributor account not found");
        u64::from_le_bytes(account.data[0..8].try_into().unwrap())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CU Summary (run with: cargo test -- --nocapture --test-threads=1)
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn zz_cu_summary() {
        let results = CU_RESULTS.lock().unwrap();
        if results.is_empty() {
            println!("No CU results recorded (run tests individually).");
            return;
        }
        println!("\n=== Compute Unit Summary ===");
        for (name, cu) in results.iter() {
            println!("  {:<40} {:>8} CUs", name, cu);
        }
    }
}
