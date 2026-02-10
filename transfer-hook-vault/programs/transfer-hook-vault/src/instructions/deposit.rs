use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::{MintTo, MintToChecked, mint_to_checked}, token_interface::{Mint, TokenAccount, TokenInterface}
};
// Option 1: Use System Program CPI
use anchor_lang::system_program::{transfer, Transfer};


use crate::{
    constatnt::{USER, VAULT},
    error::MyError,
    state::{User, Vault},
};

#[derive(Accounts)]
pub struct Deposit<'a> {
    #[account(mut)]
    pub owner: Signer<'a>,

    #[account(mut)]
    pub mint: InterfaceAccount<'a, Mint>,

    #[account(
        mut,
     seeds=[VAULT.as_bytes(),vault.admin.key().as_ref()],
     bump,
 )]
    pub vault: Account<'a, Vault>,

    #[account(mut ,seeds=[USER.as_bytes(),owner.key().as_ref()], bump)]
    pub user: Account<'a, User>,

    #[account(
      mut,
      associated_token::mint = mint,
      associated_token::authority = owner,
      associated_token::token_program = token_program
    )]
    pub owner_ata: InterfaceAccount<'a, TokenAccount>,

    pub associated_token_program: Program<'a, AssociatedToken>,
    pub token_program: Interface<'a, TokenInterface>,
    pub system_program: Program<'a, System>,
}

impl<'a> Deposit<'a> {
    pub fn deposit(&mut self, amount: u64)->Result<()> {
        if !self.user.address.key().eq(&self.owner.key()) {
            MyError::Unauthorized;
        }
        
       
     
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.owner.to_account_info(),
                    to: self.vault.to_account_info(),
                },
            ),
            amount,
        )?;

        let vault_sate_key = self.vault.admin.key();

        let seeds = &[
            VAULT.as_bytes(),
            vault_sate_key.as_ref(),
            &[self.vault.bump],
        ];

        let signer_seed = &[&seeds[..]];

        mint_to_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintToChecked {
                    mint: self.mint.to_account_info(),
                    authority: self.vault.to_account_info(),
                    to: self.owner_ata.to_account_info(),
                },
                signer_seed,
            ),
            amount,
            self.mint.decimals,
        )
        ?;
        Ok(())
    }
}
