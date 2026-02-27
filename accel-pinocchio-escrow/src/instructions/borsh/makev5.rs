use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::state::Escrow4;

pub fn process_makev5_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, escrow4_account, maker_ata, escrow4_ata, system_program, token_program, _associated_token_program @ ..] =
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
    let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];

    let escrow4_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow4_account_pda, *escrow4_account.address().as_array());

    let amount_to_receive = unsafe { *(data.as_ptr().add(1) as *const u64) };
    let amount_to_give = unsafe { *(data.as_ptr().add(9) as *const u64) };

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    unsafe {
        if escrow4_account.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: escrow4_account,
                lamports: Rent::get()?.try_minimum_balance(Escrow4::LEN)?,
                space: Escrow4::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[seeds.clone()])?;

           
        } else {
            return Err(ProgramError::IllegalOwner);
        }
    }

    let mut dummy = Escrow4::default();
       dummy.set_inner(
           escrow4_account,
           maker.address(),
           mint_a.address(),
           mint_b.address(),
           amount_to_receive.to_le_bytes(),
           amount_to_give.to_le_bytes(),
           data[0],
       )?;
    pinocchio_associated_token_account::instructions::Create {
        funding_account: maker,
        account: escrow4_ata,
        wallet: escrow4_account,
        mint: mint_a,
        token_program: token_program,
        system_program: system_program,
    }
    .invoke()?;

    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: escrow4_ata,
        authority: maker,
        amount: amount_to_give,
    }
    .invoke()?;

    Ok(())
}
