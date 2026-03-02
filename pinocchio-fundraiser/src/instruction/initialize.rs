use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{
        clock::{ Clock},
        rent::Rent,
        Sysvar,
    },
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::{constant::MIN_AMOUNT_TO_RAISE, state::Fundraiser};

pub fn process_initialize_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    //amount u64 duration u8 -> u8-> 9
    let [maker, mint_to_raise, fundraiser, vault, system_program, token_program, _associated_token_program] =
        accounts
    else {
        return Err(pinocchio::error::ProgramError::NotEnoughAccountKeys);
    };

    //list of account contraints.
    //
    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let bump = data[0];
    let seed = [b"fundraiser".as_ref(), maker.address().as_ref(), &[bump]];
    let _seeds = &seed[..];

    let escrow4_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow4_account_pda, *fundraiser.address().as_array());

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    unsafe {
        if fundraiser.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: fundraiser,
                lamports: Rent::get()?.try_minimum_balance(Fundraiser::LEN)?,
                space: Fundraiser::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[seeds.clone()])?;
        } else {
            return Err(ProgramError::IllegalOwner);
        }
    }

    pinocchio_associated_token_account::instructions::Create {
        token_program,
        system_program,
        funding_account: maker,
        account: vault,
        mint: mint_to_raise,
        wallet: fundraiser,
    }
    .invoke()?;

    if data.len() < 9 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let amount = unsafe { *(data.as_ptr().add(1) as *const u64) };
    let duration = unsafe { *(data.as_ptr().add(9) as *const u8) };

    let mint_data = mint_to_raise.try_borrow()?;
    let decimals = mint_data[44];

    let min_amount = (MIN_AMOUNT_TO_RAISE as u64)
        .checked_pow(decimals as u32)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if amount <= min_amount {
        return Err(ProgramError::Custom(12));
    }

    /*    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: u8,
    pub bump: u8, */

    let clock = Clock::get()?;
    // let time_started: i64 = clock.unix_timestamp;

    let mut fund_data = fundraiser.try_borrow_mut()?;
    let state = Fundraiser {
        maker: *maker.address().as_array(),
        mint_to_raise: *mint_to_raise.address().as_array(),
        amount_to_raise: amount.to_le_bytes(),
        current_amount: 0u64.to_le_bytes(),
        time_started: clock.unix_timestamp.to_le_bytes(),
        duration,
        bump: bump[0],
    };

    unsafe {
        // Write the whole struct into the account bytes in one shot — no index math, no copies field by field
        core::ptr::write(fund_data.as_mut_ptr() as *mut Fundraiser, state);
    }

    Ok(())
}
