use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::instructions::{CloseAccount, Transfer};

pub fn process_cancel_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [maker,maker_ata_a, escrow_account, escrow_ata, _system_program, token_program, _associated_token_program @ ..] =
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
    let (bump,amount) = {
        let escrow_data = escrow_account.try_borrow_mut()?;
        (escrow_data[112],  escrow_data[104..112].try_into().unwrap())
    };

    let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];

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

    
    Transfer {
        from: &escrow_ata,
        to: &maker_ata_a,
        authority: &escrow_account,
        amount: u64::from_le_bytes(amount),
    }
    .invoke_signed(&[seeds.clone()])?;

    CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&[seeds])?;

    //still have doubts.
    //to close the account dran full balance.
    let source_lamports = escrow_account.lamports();
    maker.set_lamports(maker.lamports() + source_lamports);
    escrow_account.set_lamports(0);
    escrow_account.resize(0)?;
    escrow_account.close()?;

    Ok(())
}
