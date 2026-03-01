use pinocchio::{AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError, sysvars::{Sysvar, clock::Clock}};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::Transfer, state::Mint};

use crate::constant::SECONDS_TO_DAYS;

pub fn process_refund_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    //amount u64 duration u8 -> u8-> 9
    let [contributor, maker, mint_to_raise, fundraiser, contributor_account, contributor_ata, vault, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(pinocchio::error::ProgramError::NotEnoughAccountKeys);
    };

    //list of account contraints.
    //
    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !Mint::LEN == mint_to_raise.data_len() {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let mut fundraiser_data = fundraiser.try_borrow_mut()?;

    let fundraiser_maker = fundraiser_data[0..32].as_ref();

    let bump = fundraiser_data[90];
    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];

    let fundraiser_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(fundraiser_pda, *fundraiser.address().as_array());

    
    let mut contributor_ata_data = contributor_ata.try_borrow_mut()
        ?;
    
    let mut vault_data = vault.try_borrow_mut()?;
    let contributor_account_data = contributor_account.try_borrow_mut()?;
    
    
    //progrma. 
    // 
    let current_time = Clock::get()?.unix_timestamp;
    
     let fundraiser_duration = fundraiser_data[32 + 32 + 8 + 8 + 8];
     
     
     let fundraiser_time_started = i64::from_le_bytes(fundraiser_data[80..88].try_into().unwrap());
    
if 
       ! (fundraiser_duration >= ((current_time - fundraiser_time_started) / SECONDS_TO_DAYS) as u8){
//        crate::FundraiserError::FundraiserNotEnded
return Err(ProgramError::Custom(100))
        }
        
        let fundraiser_amount_to_raise =
            u64::from_le_bytes(fundraiser_data[64..72].try_into().unwrap());
        let vault_amount =
            u64::from_le_bytes(vault_data[64..72].try_into().unwrap());
        if
           ! (vault_amount <fundraiser_amount_to_raise){
                
         //   crate::FundraiserError::TargetMet
         return Err(ProgramError::Custom(101))

            }
            
            
            let contributor_account_amount =
                u64::from_le_bytes(contributor_account_data[0..8].try_into().unwrap());
        
            
            let bump = [fundraiser_data[90]];
            let seed = [
                Seed::from(b"fundraiser"),
                Seed::from(maker.address().as_ref()),
                Seed::from(&bump),
            ];
            let seeds = Signer::from(&seed);
            Transfer{
                from:vault,
                to:contributor_ata,
                authority:fundraiser,
                amount:contributor_account_amount
            }.invoke_signed(&[seeds])?;
            
            
            let fundraiser_current_amaount =
                u64::from_le_bytes(fundraiser_data[72..80].try_into().unwrap());

    fundraiser_data[72..80].copy_from_slice(
        fundraiser_current_amaount
            .checked_sub(contributor_account_amount)
            .unwrap()
            .to_le_bytes()
            .as_ref(),
    );
    

    contributor_account.close()?;
    Ok(())
}
