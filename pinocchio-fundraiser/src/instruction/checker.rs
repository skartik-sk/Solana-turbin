use pinocchio::{AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError};
use pinocchio_pubkey::derive_address;

use pinocchio_token::{instructions::{ Transfer}, state::Mint};

/*use anchor_lang::prelude::*;





*/
pub fn process_checker_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    //amount u64 duration u8 -> u8-> 9
    let [maker,mint_to_raise,fundraiser,vault,maker_ata,system_program,token_program,associated_token_program]= accounts else {
        return Err(pinocchio::error::ProgramError::NotEnoughAccountKeys);
    };

    //list of account contraints. 
    // 
    if !maker.is_signer(){
       return  Err(ProgramError::MissingRequiredSignature);
    }
    if !Mint::LEN== mint_to_raise.data_len(){
        return Err(ProgramError::AccountDataTooSmall)
    }

    let vault_data = vault.try_borrow()?;
    let vault_token_amount = {
        let slice = vault_data.get(64..72).ok_or(ProgramError::AccountDataTooSmall)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(slice);
        u64::from_le_bytes(arr)
    };

    let fundraiser_data = fundraiser.try_borrow()?;
    let fundraiser_amount = {
        let slice = fundraiser_data.get(64..72).ok_or(ProgramError::AccountDataTooSmall)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(slice);
        u64::from_le_bytes(arr)
    };

    if vault_token_amount < fundraiser_amount {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let bump = fundraiser_data[89];
    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];


    let fundraiser_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(fundraiser_pda, *fundraiser.address().as_array());

   

    //inti if neede for maket ata
    if maker_ata.lamports() > 0 {
        //already intilize do nothing.
    } else {
        pinocchio_associated_token_account::instructions::Create {
            funding_account: maker,
            account: maker_ata,
            wallet: maker,
            mint: mint_to_raise,
            token_program: token_program,
            system_program: system_program,
        }
        .invoke()?;
    }
    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    Transfer{
        from:vault,
        to:maker_ata,
        authority:fundraiser,
        amount:vault_token_amount
    }.invoke_signed(&[seeds])?;
             fundraiser.close()?;
    Ok(())}

