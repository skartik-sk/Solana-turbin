use pinocchio::{AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError};
use pinocchio_log::log;
use pinocchio_pubkey::derive_address;

use pinocchio_token::{instructions::{ Transfer}, state::Mint};

/*use anchor_lang::prelude::*;





*/
pub fn process_checker_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    //amount u64 duration u8 -> u8-> 9
    let [maker,mint_to_raise,fundraiser,vault,maker_ata,system_program,token_program,_associated_token_program]= accounts else {
        return Err(pinocchio::error::ProgramError::NotEnoughAccountKeys);
    };

    //list of account contraints.
    //
    if !maker.is_signer(){
       return  Err(ProgramError::MissingRequiredSignature);
    }
    if !(Mint::LEN== mint_to_raise.data_len()){
        return Err(ProgramError::AccountDataTooSmall)
    }


    let vault_token_amount ={
    let vault_data = vault.try_borrow()?;
        
        u64::from_le_bytes(vault_data[64..72].try_into().unwrap())
        };



    let (fundraiser_amount_raise,bump) = {
       let fundraiser_data =  fundraiser.try_borrow()?;
        ( u64::from_le_bytes(fundraiser_data[64..72].try_into().unwrap()),fundraiser_data[89])
    };
       


  
    if vault_token_amount < fundraiser_amount_raise {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];


    let fundraiser_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(fundraiser_pda, *fundraiser.address().as_array());
    


    //inti if neede for maket ata
    if maker_ata.lamports() <= 0 {

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
        Seed::from(maker.address().as_ref()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);
    Transfer{
        from:vault,
        to:maker_ata,
        authority:fundraiser,
        amount:vault_token_amount
    }.invoke_signed(&[seeds])?;

    
    let source_lamports = fundraiser.lamports();
        maker.set_lamports(maker.lamports() + source_lamports);
        fundraiser.set_lamports(0);
        fundraiser.resize(0)?;
        fundraiser.close()?;
    log!("readerd here end");
   
    Ok(())}
