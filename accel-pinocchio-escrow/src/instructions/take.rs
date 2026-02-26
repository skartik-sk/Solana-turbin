use pinocchio::{
    AccountView, ProgramResult, account::RuntimeAccount, cpi::{Seed, Signer}, error::ProgramError, sysvars::{Sysvar, rent::Rent}
    
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::{CloseAccount, Transfer}, state::Mint};


use crate::{state::Escrow, util::ATA};

pub fn process_take_instruction(
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {

    let [
        taker,
        _maker,
        _mint_a,
        _mint_b,
        escrow_account,
        taker_ata_b,
        taker_ata_a,
        maker_ata_b,
        escrow_ata,
        system_program,
        token_program,
        _associated_token_program@ ..
    ] = accounts else {
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
    let maker: [u8; 32] = escrow_data[0..31].try_into().unwrap();
    let mint_a: [u8; 32] = escrow_data[32..63].try_into().unwrap();
    let mint_b: [u8; 32] = escrow_data[64..95].try_into().unwrap();
    let amount_to_receive: [u8; 8] = escrow_data[96..103].try_into().unwrap();
    let amount_to_give: [u8; 8] = escrow_data[104..111].try_into().unwrap();
    let bump: u8 = escrow_data[112].try_into().unwrap();

    if (_mint_a.address().as_ref() != mint_a.as_ref())|| (_mint_b.address().as_ref()!= mint_b.as_ref()) || (_maker.address().as_ref() != maker.as_ref()) {
        return Err(ProgramError::InvalidAccountData)
    }
    
    let seed = [b"escrow".as_ref(), maker.as_ref(), &[bump]];
    let seeds = &seed[..];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow_account_pda, *escrow_account.address().as_array());

    if !taker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature)
    }
    
    let taker_ata_b_state = pinocchio_token::state::TokenAccount::from_account_view(&taker_ata_b)?;
    if taker_ata_b_state.owner() != taker.address() {
        return Err(ProgramError::IllegalOwner);
    }
    if taker_ata_b_state.mint().as_ref() != mint_b.as_ref() {
        return Err(ProgramError::InvalidAccountData);
    } 

        
    ATA::init_if_needed(
        _mint_a,
        &token_program,
        &system_program,
        &taker,
        &taker,
        &taker_ata_a
    )?;
    ATA::init_if_needed(
        _mint_b,
        &token_program,
        &system_program,
        &_maker,
        &taker,
        &maker_ata_b
    )?;


    let bump = [bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(_maker.address().as_array()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);
    
    
Transfer{
    from:taker_ata_b,
    to:maker_ata_b,
    authority:taker,
    amount:u64::from_le_bytes(amount_to_receive)
}.invoke()?;


Transfer{
    from:escrow_ata,
    to:taker_ata_a,
    authority:taker,
    amount:u64::from_le_bytes(amount_to_give)
}.invoke_signed(&[seeds.clone()])?;


CloseAccount{
    account:escrow_ata,
    destination:_maker,
    authority:escrow_account
}.invoke_signed(&[seeds])?;


//still have doubts.
//to close the account dran full balance. 


Transfer{
    from:escrow_account,
    to:_maker,
    amount:escrow_account.lamports(),
    authority:system_program
}.invoke()?;

    Ok(())
}