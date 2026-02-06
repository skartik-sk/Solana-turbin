#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod instructions;
mod state;

use instructions::*;

mod constant;
use constant::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::{
    instruction::{
        ExecuteInstruction, 
        InitializeExtraAccountMetaListInstruction
    },
};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

declare_id!("2wYQxAzhZYvxQbsyPkv9tgv1FRw8fw5jywL3yhKyLWaJ");

#[program]
pub mod whitelist_transfer_hook {
    use anchor_lang::system_program::{CreateAccount, create_account};

    use crate::instruction::InitMint;

    use super::*;

    pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
        ctx.accounts.initialize_whitelist(ctx.bumps)
    }

    pub fn add_to_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.add_to_whitelist(user,&ctx.bumps)
    }

    pub fn remove_from_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist(user)
    }
    
    pub fn init_mint(ctx: Context<TokenFactory>, decimal:u8) -> Result<()> {
        ctx.accounts.init_mint(&ctx.bumps,decimal)
    }

    pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {

        msg!("Initializing Transfer Hook...");

        // Get the extra account metas for the transfer hook
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        msg!("Extra Account Metas: {:?}", extra_account_metas);
        msg!("Extra Account Metas Length: {}", extra_account_metas.len());
        // let account_size = ExtraAccountMetaList::size_of(extra_account_metas.len())
        //             .map_err(|_| ProgramError::AccountDataTooSmall)?;
        // let lamports = Rent::get()?.minimum_balance(
        //     account_size); 
        
        // let mint_key = &ctx.accounts.mint.key();
        // let signer_seeds : &[&[&[u8]]]= &[&[
        //             b"extra-account-metas",
        //             mint_key.as_ref(),
        //             &[ctx.bumps.extra_account_meta_list],
        //         ]];
        
        // create_account(
        //             CpiContext::new(
        //                 ctx.accounts.system_program.to_account_info(),
        //                 CreateAccount {
        //                     from: ctx.accounts.payer.to_account_info(),
        //                     to: ctx.accounts.extra_account_meta_list.to_account_info(),
        //                 },
        //             )
        //             .with_signer(signer_seeds),
        //             lamports,
        //             account_size as u64,
        //             &crate::ID,
        //         )?;
        
  
        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas
        ).unwrap();

        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        // Call the transfer hook logic
        msg!("trasfer-hook call");
        ctx.accounts.transfer_hook(amount)
    }
}
