#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod instructions;
mod state;
mod tests;

use instructions::*;

declare_id!("FircrADQ2wgGuvpm8qneNCfKM7o5zoHTWnDQxngpTQ3J");

#[program]
pub mod anchor_escrow {

    use crate::program::AnchorEscrow;

    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time - ctx.accounts.escrow.created_at >= (5 * 24 * 60 * 60),
            CustomError::EscrowLocked
        );
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault();

        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    #[msg("You can take escrow only after 5 days")]
    EscrowLocked,
}
