use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::{state::Escrow1, util::ATA};

pub fn process_takev2_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [taker, _maker, _mint_a, _mint_b, escrow_account, taker_ata_b, taker_ata_a, maker_ata_b, escrow_ata, system_program, token_program, _associated_token_program @ ..] =
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
    let escrow: Escrow1 = {
            let escrow_data = escrow_account.try_borrow_mut()?;
            wincode::deserialize::<Escrow1>(&*escrow_data)
                .map_err(|_| ProgramError::InvalidAccountData)?
        };
    
        let maker           = escrow._maker();
        let mint_a          = escrow._mint_a();
        let mint_b          = escrow._mint_b();
        let amount_to_receive = escrow._amount_to_receive();
        let amount_to_give  = escrow._amount_to_give();
        let bump            = escrow.bump;


    if (_mint_a.address().as_ref() != mint_a.as_ref())
        || (_mint_b.address().as_ref() != mint_b.as_ref())
        || (_maker.address().as_ref() != maker.as_ref())
    {
        return Err(ProgramError::InvalidAccountData);
    }

    {
    let seed = [b"escrow".as_ref(), maker.as_ref(), &[bump]];
    let _seeds = &seed[..];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    assert_eq!(escrow_account_pda, *escrow_account.address().as_array());

    if !taker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
        let taker_ata_b_state =
            pinocchio_token::state::TokenAccount::from_account_view(&taker_ata_b)?;
        if taker_ata_b_state.owner() != taker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if taker_ata_b_state.mint().as_ref() != mint_b.as_ref() {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    log!("reached Here");
    ATA::init_if_needed(
        _mint_b,
        &token_program,
        &system_program,
        &_maker,
        &taker,
        &maker_ata_b,
    )?;

    log!("reached Here1");

    ATA::init_if_needed(
        _mint_a,
        &token_program,
        &system_program,
        &taker,
        &taker,
        &taker_ata_a,
    )?;
    log!("reached Here2");

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(_maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    Transfer {
        from: &taker_ata_b,
        to: &maker_ata_b,
        authority: &taker,
        amount: amount_to_receive,
    }
    .invoke()?;

    log!("reached Here3");
    Transfer {
        from: &escrow_ata,
        to: &taker_ata_a,
        authority: &escrow_account,
        amount: (amount_to_give),
    }
    .invoke_signed(&[seeds.clone()])?;

    log!("reached Here4");
    CloseAccount {
        account: escrow_ata,
        destination: _maker,
        authority: escrow_account,
    }
    .invoke_signed(&[seeds.clone()])?;

    log!("reached Here5");
    //still have doubts.
    //to close the account dran full balance.
    // escrow_account.try_borrow_mut()
    /*  Transfer {
          from: escrow_account,
          to: maker,
          amount: escrow_account.lamports(),
          authority: system_program,
      }
      .invoke()?;
      
      
      unsafe{
      escrow_account.close
      }
      */
      // drop(data);
      // 
      // /// close
      let source_lamports = escrow_account.lamports();
           _maker.set_lamports(_maker.lamports()+source_lamports);
           escrow_account.set_lamports(0);
           escrow_account.resize(0)?;
           escrow_account.close()?;
    //     escrow_account.resize(0) ?;
    //   pinocchio_system::instructions::Transfer {
    //         from: escrow_account,
    //         to: _maker,
    //         lamports: escrow_account.lamports(),
    //     }
    // .invoke_signed(&[seeds])?;

    //     escrow_account.close()?;
    // unsafe {
    //     _maker.lamports() += escrow_account.lamports();
    //     escrow_account.close();
    //     escrow_account.resize(0) ?;
    // }
    log!("reached Here6");

    Ok(())
}
