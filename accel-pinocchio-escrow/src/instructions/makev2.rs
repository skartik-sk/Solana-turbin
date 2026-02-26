use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::state::Escrow1;

pub fn process_makev2_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, escrow1_account, maker_ata, escrow1_ata, system_program, token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    {
        let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(&maker_ata)?;
        if maker_ata_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if maker_ata_state.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let bump = data[0];
    let seed = [b"escrow1".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];

    let escrow1_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow1_account_pda, *escrow1_account.address().as_array());

    let amount_to_receive = unsafe { *(data.as_ptr().add(1) as *const u64) };
    let amount_to_give = unsafe { *(data.as_ptr().add(9) as *const u64) };

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow1"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    unsafe {
        if escrow1_account.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: escrow1_account,
                lamports: Rent::get()?.try_minimum_balance(Escrow1::LEN)?,
                space: Escrow1::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[seeds.clone()])?;

            {
                let escrow1_state = Escrow1::from_account_info(escrow1_account)?;
                // wincode::serialize(&Escrow1{maker:maker.address().as_array(),
                //     mint_a:mint_a.address().as_array(),
                //     mint_b:mint_b.address().as_array(),
                //     amount_to_receive:amount_to_receive.to_le_bytes(),
                //     amount_to_give:amount_to_give.to_le_bytes(),
                //     bump:data[0]
                // });
                escrow1_state.set_maker(maker.address());
                escrow1_state.set_mint_a(mint_a.address());
                escrow1_state.set_mint_b(mint_b.address());
                escrow1_state.set_amount_to_receive(amount_to_receive);
                escrow1_state.set_amount_to_give(amount_to_give);
                escrow1_state.bump = data[0];
            }
        } else {
            return Err(ProgramError::IllegalOwner);
        }
    }

    pinocchio_associated_token_account::instructions::Create {
        funding_account: maker,
        account: escrow1_ata,
        wallet: escrow1_account,
        mint: mint_a,
        token_program: token_program,
        system_program: system_program,
    }
    .invoke()?;

    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: escrow1_ata,
        authority: maker,
        amount: amount_to_give,
    }
    .invoke()?;

    Ok(())
}
