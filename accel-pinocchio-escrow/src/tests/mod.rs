#[cfg(test)]
mod tests {

    use litesvm::LiteSVM;
    use litesvm_token::{
        spl_token::{self},
        CreateAssociatedTokenAccount, CreateMint, MintTo,
    };
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::get_associated_token_address;
    use std::{path::PathBuf, sync::Mutex};

    const PROGRAM_ID: &str = "9piQZir4QXTh76Xt9HwVSFtisTud8paBcWWeea6qCJxS";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    static CU_RESULTS: Mutex<Vec<(&'static str, u64)>> = Mutex::new(Vec::new());

    fn record_cu(name: &'static str, cu: u64) {
        CU_RESULTS.lock().unwrap().push((name, cu));
    }
    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn setup() -> (LiteSVM, Keypair, Keypair) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        let taker = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file
        // println!("The path is!! {}", env!("CARGO_MANIFEST_DIR"));
        let so_path = PathBuf::from("/Users/singupallikartik/Developer/Q1_Acc_skartik-sk/accel-pinocchio-escrow/target/sbpf-solana-solana/release/escrow.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        svm.add_program(program_id(), &program_data)
            .expect("Failed to add program");

        (svm, payer, taker)
    }

    fn helper(
        mut svm: &mut LiteSVM,
        payer: &Keypair,
        taker: &Keypair,
    ) -> (
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        (Pubkey, u8),
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
    ) {
        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint_a = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        // println!("Mint A: {}", mint_a);

        let mint_b = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        // println!("Mint B: {}", mint_b);
        // Create the maker's associated token account for Mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
            .owner(&payer.pubkey())
            .send()
            .unwrap();
        // println!("Maker ATA A: {}\n", maker_ata_a);

        let maker_ata_b = get_associated_token_address(&payer.pubkey(), &mint_b);
        // println!("Maker ATA B: {}\n", maker_ata_b);

        let taker_ata_a = get_associated_token_address(&taker.pubkey(), &mint_a);
        // println!("Taker ATA A: {}\n", taker_ata_a);

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        // println!("Taker ATA B: {}\n", taker_ata_b);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        // println!("Escrow PDA: {}\n", escrow.0);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = spl_associated_token_account::get_associated_token_address(
            &escrow.0, // owner will be the escrow PDA
            &mint_a,   // mint
        );
        // println!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        (
            program_id,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
            system_program,
            token_program,
            associated_token_program,
        )
    }

    #[test]
    pub fn test_make_instruction() {
        let (mut svm, payer, taker) = setup();

        let (
            program_id,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
            system_program,
            token_program,
            associated_token_program,
        ) = helper(&mut svm, &payer, &taker);

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000; // 500 tokens with 6 decimal places
        let bump: u8 = escrow.1;

        //// println!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![0u8], // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        println!("\n\nMake transaction sucessfull");
        record_cu("make", tx.compute_units_consumed);
    }

    #[test]
    pub fn test_take_instruction() {
        let (mut svm, payer, taker) = setup();

        let (
            program_id,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
            system_program,
            token_program,
            associated_token_program,
        ) = helper(&mut svm, &payer, &taker);

        //---------------------Make----------------------------

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000; // 500 tokens with 6 decimal places
        let bump: u8 = escrow.1;

        //// println!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![0u8], // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        // println!("\n\nMake transaction sucessfull");
        // println!("CUs Consumed: {}", tx.compute_units_consumed);
        // assert_eq!(svm.get_balance(&maker_ata_a),)

        //---------------------Take----------------------------

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_b, &taker_ata_b, 1000000000)
            .send()
            .unwrap();
        let bump: u8 = escrow.1;

        //println!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let take_data = [
            vec![1u8], // Discriminator for "Make" instruction
                       // bump.to_le_bytes().to_vec(),
                       // amount_to_receive.to_le_bytes().to_vec(),
                       // amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let take_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(taker_ata_b, false),
                AccountMeta::new(taker_ata_a, false),
                AccountMeta::new(maker_ata_b, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: take_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&taker], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        println!("\n\n Take transaction sucessfull");
        record_cu("Take", tx.compute_units_consumed);
    }

    #[test]
    pub fn test_cancel_instruction() {
        let (mut svm, payer, taker) = setup();

        let (
            program_id,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
            system_program,
            token_program,
            associated_token_program,
        ) = helper(&mut svm, &payer, &taker);

        //---------------------Make----------------------------

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000; // 500 tokens with 6 decimal places
        let bump: u8 = escrow.1;

        //println!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![0u8], // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        // println!("\n\nMake transaction sucessfull");
        // record_cu("make", tx.compute_units_consumed);
        // assert_eq!(svm.get_balance(&maker_ata_a),)

        //---------------------Take----------------------------

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account

        // Create the "Make" instruction to deposit tokens into the escrow
        let cancel_data = [
            vec![2u8], // Discriminator for "Make" instruction
                       // bump.to_le_bytes().to_vec(),
                       // amount_to_receive.to_le_bytes().to_vec(),
                       // amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let cancel_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: cancel_data,
        };

        // Create and send the transaction containing the "cancel" instruction
        let message = Message::new(&[cancel_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        println!("\n\n Cancel transaction sucessfull");
        record_cu("Cancel", tx.compute_units_consumed);
    }

    #[test]
    pub fn test_makev2_instruction() {
        let (mut svm, payer, taker) = setup();

        let (
            program_id,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
            system_program,
            token_program,
            associated_token_program,
        ) = helper(&mut svm, &payer, &taker);

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000; // 500 tokens with 6 decimal places
        let bump: u8 = escrow.1;

        //println!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![3u8], // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        println!("\n\nMakeV2(Wincode) transaction sucessfull");
        record_cu("make/wincode", tx.compute_units_consumed);
    }

    #[test]
    pub fn test_takev2_instruction() {
        let (mut svm, payer, taker) = setup();

        let (
            program_id,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
            system_program,
            token_program,
            associated_token_program,
        ) = helper(&mut svm, &payer, &taker);

        //---------------------Make----------------------------

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000; // 500 tokens with 6 decimal places
        let bump: u8 = escrow.1;

        //println!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![3u8], // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        // println!("\n\nMake transaction sucessfull");
        // println!("CUs Consumed: {}", tx.compute_units_consumed);
        // assert_eq!(svm.get_balance(&maker_ata_a),)

        //---------------------Take----------------------------

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_b, &taker_ata_b, 1000000000)
            .send()
            .unwrap();
        let bump: u8 = escrow.1;

        //println!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let take_data = [
            vec![4u8], // Discriminator for "Make" instruction
                       // bump.to_le_bytes().to_vec(),
                       // amount_to_receive.to_le_bytes().to_vec(),
                       // amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let take_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(taker_ata_b, false),
                AccountMeta::new(taker_ata_a, false),
                AccountMeta::new(maker_ata_b, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: take_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&taker], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        println!("\n\n TakeV2(wincode) transaction sucessfull");
        record_cu("Take/wincode", tx.compute_units_consumed);
    }
    #[test]
    // Named "zz_" so it sorts last alphabetically and runs after all others.
    // Run with: cargo test -- --nocapture --test-threads=1
    #[test]
    fn zz_cu_summary() {
        // Run with: cargo test -- --nocapture --test-threads=1
        //
        // Naming convention for record_cu: "group/variant"
        //   e.g.  record_cu("make/base", cu)
        //         record_cu("make/wincode", cu)
        //         record_cu("make/borsh", cu)   ← just add this, no other change needed
        //
        // The summary auto-groups by the prefix and diffs every variant
        // against the "base" variant of the same group.
        let results = CU_RESULTS.lock().unwrap();
        if results.is_empty() {
            println!("No CU results recorded.");
            return;
        }

        // ── 1. Build an ordered map: group → [(variant, cu)] ────────────────
        // Use a Vec to preserve insertion order of groups.
        let mut groups: Vec<(&str, Vec<(&str, u64)>)> = Vec::new();

        for (name, cu) in results.iter() {
            let (group, variant) = match name.split_once('/') {
                Some((g, v)) => (g, v),
                None => (*name, "base"), // no slash → treat whole name as group/base
            };

            if let Some(entry) = groups.iter_mut().find(|(g, _)| *g == group) {
                entry.1.push((variant, *cu));
            } else {
                groups.push((group, vec![(variant, *cu)]));
            }
        }

        // ── 2. Figure out column widths ──────────────────────────────────────
        let col = groups
            .iter()
            .flat_map(|(g, variants)| {
                variants.iter().map(|(v, _)| {
                    if *v == "base" {
                        g.len()
                    } else {
                        g.len() + v.len() + 3
                    }
                })
            })
            .max()
            .unwrap_or(0)
            .max(15);

        let bar1 = "─".repeat(col + 2);
        let bar2 = "─".repeat(14);
        let bar3 = "─".repeat(14);

        // ── 3. Print ─────────────────────────────────────────────────────────
        println!("\n");
        println!("┌{}┬{}┬{}┐", bar1, bar2, bar3);
        println!(
            "│ {:<col$} │ {:>12} │ {:>12} │",
            "Instruction",
            "CUs Consumed",
            "vs base",
            col = col
        );

        for (group, variants) in &groups {
            println!("├{}┼{}┼{}┤", bar1, bar2, bar3);

            // find the base CU for this group (variant == "base")
            let base_cu = variants
                .iter()
                .find(|(v, _)| *v == "base")
                .map(|(_, cu)| *cu);

            for (variant, cu) in variants {
                let label = if *variant == "base" {
                    format!("{}", group)
                } else {
                    format!("{} ({})", group, variant)
                };

                let delta_str = match base_cu {
                    _ if *variant == "base" => "  (baseline)".to_string(),
                    Some(b) => {
                        let d = *cu as i64 - b as i64;
                        if d > 0 {
                            format!("      ▲ +{}", d)
                        } else if d < 0 {
                            format!("      ▼ {}", d)
                        } else {
                            "   no change".to_string()
                        }
                    }
                    None => "           —".to_string(),
                };

                println!(
                    "│ {:<col$} │ {:>12} │ {:>12} │",
                    label,
                    cu,
                    delta_str,
                    col = col
                );
            }
        }

        println!("└{}┴{}┴{}┘", bar1, bar2, bar3);
        println!();
    }
}
