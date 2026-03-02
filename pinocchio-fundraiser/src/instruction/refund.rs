use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::Transfer, state::Mint};

use crate::constant::SECONDS_TO_DAYS;

pub fn process_refund_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    //amount u64 duration u8 -> u8-> 9
    let [contributor, maker, mint_to_raise, fundraiser, contributor_account, contributor_ata, vault, _system_program, _token_program, _associated_token_program] =
        accounts
    else {
        return Err(pinocchio::error::ProgramError::NotEnoughAccountKeys);
    };

    //list of account contraints.
    //
    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !(Mint::LEN == mint_to_raise.data_len()) {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let (
        bump,
        fundraiser_duration,
        fundraiser_time_started,
        fundraiser_amount_to_raise,
        fundraiser_current_amaount,
        seed_bump,
    ) = {
        let fundraiser_data = fundraiser.try_borrow_mut()?;

        (
            fundraiser_data[89],
            fundraiser_data[32 + 32 + 8 + 8 + 8],
            i64::from_le_bytes(fundraiser_data[80..88].try_into().unwrap()),
            u64::from_le_bytes(fundraiser_data[64..72].try_into().unwrap()),
            u64::from_le_bytes(fundraiser_data[72..80].try_into().unwrap()),
            [fundraiser_data[89]],
        )
    };
    // let fundraiser_maker = fundraiser_data[0..32].as_ref();

    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];

    let fundraiser_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(fundraiser_pda, *fundraiser.address().as_array());

    log!("Reached here 0");
    // let mut contributor_ata_data = contributor_ata.try_borrow_mut()?;



    //progrma.
    //
    let current_time = Clock::get()?.unix_timestamp;

    if !(fundraiser_duration >= ((current_time - fundraiser_time_started) / SECONDS_TO_DAYS) as u8)
    {
        //        crate::FundraiserError::FundraiserNotEnded
        return Err(ProgramError::Custom(300));
    }

    let vault_amount = {
    let vault_data = vault.try_borrow()?;
        u64::from_le_bytes(vault_data[64..72].try_into().unwrap())};
    if !(vault_amount < fundraiser_amount_to_raise) {
        //   crate::FundraiserError::TargetMet
        return Err(ProgramError::Custom(301));
    }

    let contributor_account_amount ={
       let contributor_account_data=  contributor_account.try_borrow()?;
        u64::from_le_bytes(contributor_account_data[0..8].try_into().unwrap())
        };

    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_ref()),
        Seed::from(&seed_bump),
    ];
    let seeds = Signer::from(&seed);
    log!("Reached here 3");
    Transfer {
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        amount: contributor_account_amount,
    }
    .invoke_signed(&[seeds])?;
    log!("Reached here 4");
    let mut fundraiser_data = fundraiser.try_borrow_mut()?;

    fundraiser_data[72..80].copy_from_slice(
        fundraiser_current_amaount
            .checked_sub(contributor_account_amount)
            .unwrap()
            .to_le_bytes()
            .as_ref(),
    );
    log!("Reached here 5");
    let source_lamports = contributor_account.lamports();
    maker.set_lamports(maker.lamports() + source_lamports);
    contributor_account.set_lamports(0);
    contributor_account.resize(0)?;
    contributor_account.close()?;
    log!("Reached here end");
    Ok(())
}
