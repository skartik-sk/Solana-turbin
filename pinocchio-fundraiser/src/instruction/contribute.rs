use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::Transfer, state::Mint};

use crate::constant::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS};

pub fn process_contribute_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    //amount u64 duration u8 -> u8-> 9
    let [contributor, mint_to_raise, fundraiser, contributor_account, contributor_ata, vault, _system_program, _token_program] =
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

    let mut fundraiser_data = fundraiser.try_borrow_mut()?;

    let maker = fundraiser_data[0..32].as_ref();

    let bump = fundraiser_data[89];
    let seed = [b"fundraiser".as_ref(), maker, &[bump]];
    let _seeds = &seed[..];

    let fundraiser_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(fundraiser_pda, *fundraiser.address().as_array());
    // Verify mint_to_raise matches what's in the fundraiser
    let fundraiser_mint = fundraiser_data[32..64].as_ref();
    if fundraiser_mint != mint_to_raise.address().as_ref() {
        return Err(ProgramError::Custom(101));
    }

    let contributor_bump = [data[8]];
    let contributor_seed = [b"contributor", fundraiser.address().as_ref(), contributor.address().as_ref(), &contributor_bump];
    let derived = derive_address(&contributor_seed, None, &crate::ID.to_bytes());
    assert_eq!(derived, *contributor_account.address().as_array());

    let seed = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser.address().as_ref()), Seed::from(contributor.address().as_ref()),
        Seed::from(&contributor_bump),
    ];
    let seeds = Signer::from(&seed);

    let lamports = Rent::get()?.try_minimum_balance(8)?;
    //inti if neede for maket ata
    if contributor_account.lamports() > 0 {
        //already intilize do nothing.
    } else {
        pinocchio_system::instructions::CreateAccount {
            from: contributor,
            to: contributor_account,
            space: 8,
            lamports,
            owner: &crate::ID,
        }
        .invoke_signed(&[seeds])?;
    }

    //program
    //
    //
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());

    let mint_to_raise_decimals = mint_to_raise.try_borrow()?[32 + 8];
    // Check if the amount to contribute meets the minimum amount required
    if !(amount > 1_u8.pow(mint_to_raise_decimals as u32) as u64) {
        return Err(ProgramError::Custom(102));
    }

    // Check if the amount to contribute is less than the maximum allowed contribution
    //
    //
    let fundraiser_amount_to_raise =
        u64::from_le_bytes(fundraiser_data[64..72].try_into().unwrap());
    if !(amount <= (fundraiser_amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER) {
        return Err(ProgramError::Custom(103));
    }

    /*
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: u8,
    pub bump: u8,
    */

    let fundraiser_duration = fundraiser_data[32 + 32 + 8 + 8 + 8];

    let fundraiser_time_started = i64::from_le_bytes(fundraiser_data[80..88].try_into().unwrap());
    // Check if the fundraising duration has been reached
    let current_time = Clock::get()?.unix_timestamp;
    if !(fundraiser_duration <= ((current_time - fundraiser_time_started) / SECONDS_TO_DAYS) as u8) {
        return Err(ProgramError::Custom(104));
        // crate::FundraiserError::FundraiserEnded
    }
    let mut contributor_account_data = contributor_account.try_borrow_mut()?;
    let contributor_account_amount =
        u64::from_le_bytes(contributor_account_data[0..8].try_into().unwrap());
    // Check if the maximum contributions per contributor have been reached
    if !((contributor_account_amount
        <= (fundraiser_amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER)
        && (contributor_account_amount + amount
            <= (fundraiser_amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER))
    {
        return Err(ProgramError::Custom(105));
        // FundraiserError::MaximumContributionsReached
    }

    Transfer {
        from: contributor_ata,
        authority: contributor,
        to: vault,
        amount: amount,
    }
    .invoke()?;

    let fundraiser_current_amaount =
        u64::from_le_bytes(fundraiser_data[72..80].try_into().unwrap());
    fundraiser_data[72..80].copy_from_slice(
        fundraiser_current_amaount
            .checked_add(amount)
            .unwrap()
            .to_le_bytes()
            .as_ref(),
    );

    //? - i Think
    contributor_account_data[0..8].copy_from_slice(
        contributor_account_amount
            .checked_add(amount)
            .unwrap()
            .to_le_bytes()
            .as_ref(),
    );

    Ok(())
}
