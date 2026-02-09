use anchor_lang::{prelude::*, system_program, Bump};

use crate::{
    constatnt::{USER, VAULT},
    state::{User, Vault},
};

#[derive(Accounts)]
#[instruction(address:Pubkey)]
pub struct WhitelistOperations<'a> {
    #[account(
        mut,
        address = vault.admin
    )]
    pub admin: Signer<'a>,

    #[account(

    seeds=[VAULT.as_bytes(),admin.key().as_ref()],
    bump,
)]
    pub vault: Account<'a, Vault>,

    #[account(init_if_needed,payer=admin,space = 8+User::LEN,seeds=[USER.as_bytes(),address.key().as_ref()], bump)]
    pub user: Account<'a, User>,

    pub system_program: Program<'a, System>,
}

impl<'a> WhitelistOperations<'a> {
    pub fn add_to_whitelist(
        &mut self,
        address: Pubkey,
        bump: &WhitelistOperationsBumps,
    ) -> Result<()> {
        self.user.set_inner(User {
            address,
            bump: bump.user,
        });
        Ok(())
    }

    pub fn remove_from_whitelist(&mut self, _address: Pubkey) -> Result<()> {
        self.user.close(self.admin.to_account_info())?;
        Ok(())
    }
}
