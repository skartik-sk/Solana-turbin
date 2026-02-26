use pinocchio::{
    account::RuntimeAccount,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::Mint,
};

use crate::{state::Escrow, util::ATA};

pub fn process_cancel_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, escrow_account, escrow_ata, system_program, token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // pub struct Escrow {
    //     maker: [u8; 32],
    //     mint_a: [u8; 32],
    //     mint_b: [u8; 32],
    //     amount_to_receive: [u8; 8],
    //     amount_to_give: [u8; 8],
    //     pub bump: u8,
    // }
    //[start..end)
    let escrow_data = escrow_account.try_borrow_mut()?;
    let bump: u8 = escrow_data[112].try_into().unwrap();

    let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];
    let seeds = &seed[..];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow_account_pda, *escrow_account.address().as_array());

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&[seeds])?;

    //still have doubts.
    //to close the account dran full balance.

    Transfer {
        from: escrow_account,
        to: maker,
        amount: escrow_account.lamports(),
        authority: system_program,
    }
    .invoke()?;

    Ok(())
}
