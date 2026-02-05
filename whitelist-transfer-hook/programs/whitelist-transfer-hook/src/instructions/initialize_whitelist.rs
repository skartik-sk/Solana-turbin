use anchor_lang::prelude::*;

use crate::state::{Admin};

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Admin::LEN, // 8 bytes for discriminator, 4 bytes for vector length, 1 byte for bump
        seeds = [b"admin"],
        bump
    )]
    pub token_admin: Account<'info, Admin>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeWhitelist<'info> {
    pub fn initialize_whitelist(&mut self, bumps: InitializeWhitelistBumps) -> Result<()> {
        // Initialize the whitelist with an empty address vector
        self.token_admin.set_inner(Admin { 
            address: *self.admin.key,
            bump: bumps.token_admin,
        });

        Ok(())
    }
}