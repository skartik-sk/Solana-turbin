use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::state::Escrow3;

pub fn process_makev4_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, escrow3_account, maker_ata, escrow3_ata, system_program, token_program, _associated_token_program @ ..] =
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

    let escrow3_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow3_account_pda, *escrow3_account.address().as_array());

    let amount_to_receive = unsafe { *(data.as_ptr().add(1) as *const u64) };
    let amount_to_give = unsafe { *(data.as_ptr().add(9) as *const u64) };

    // Serialize first to know the exact byte length
    let escrow3_data = Escrow3 {
        maker:             *maker.address().as_array(),
        mint_a:            *mint_a.address().as_array(),
        mint_b:            *mint_b.address().as_array(),
        amount_to_receive: amount_to_receive.to_le_bytes(),
        amount_to_give:    amount_to_give.to_le_bytes(),
        bump:              data[0],
    };
    let json_bytes = serde_json::to_vec(&escrow3_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let json_len = json_bytes.len();  // exact size, no waste, no trailing zeros

    let bump_byte = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_byte),
    ];
    let seeds = Signer::from(&seed);

    unsafe {
        if escrow3_account.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: escrow3_account,
                lamports: Rent::get()?.try_minimum_balance(json_len)?, // ← exact
                space: json_len as u64,                                 // ← exact
                owner: &crate::ID,
            }
            .invoke_signed(&[seeds.clone()])?;
        } else {
            return Err(ProgramError::IllegalOwner);
        }
    }

    // Write exact bytes — account is exactly json_len, no trailing zeros
    {
        let mut acc_data = escrow3_account.try_borrow_mut()?;
        acc_data.copy_from_slice(&json_bytes);
    }


    pinocchio_associated_token_account::instructions::Create {
        funding_account: maker,
        account: escrow3_ata,
        wallet: escrow3_account,
        mint: mint_a,
        token_program: token_program,
        system_program: system_program,
    }
    .invoke()?;

    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: escrow3_ata,
        authority: maker,
        amount: amount_to_give,
    }
    .invoke()?;

    Ok(())
}
