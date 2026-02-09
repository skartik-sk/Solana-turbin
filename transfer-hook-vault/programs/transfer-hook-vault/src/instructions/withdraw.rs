use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_2022::{BurnChecked, MintTo, MintToChecked, Transfer, burn_checked, mint_to_checked}, token_interface::{Mint, TokenAccount, TokenInterface}
};

use crate::{
    constatnt::{USER, VAULT}, error::MyError, state::{User, Vault}
};

#[derive(Accounts)]
pub struct Withdraw<'a> {
    #[account(mut)]
    pub owner: Signer<'a>,

    #[account(
        
    )]
    pub mint: InterfaceAccount<'a,Mint>,

    #[account(
     seeds=[VAULT.as_bytes(),vault.admin.key().as_ref()],
     bump,
 )]
    pub vault: Account<'a, Vault>,

    #[account(init,payer=owner,space = 8+User::LEN,seeds=[USER.as_bytes(),owner.key().as_ref()], bump)]
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

impl<'a> Withdraw<'a> {
    pub fn withdraw(&mut self, amount: u64) {
        //not used but to be sure. 
        if !self.user.address.key().eq(&self.owner.key()){
            MyError::Unauthorized;
        }
        
        **self.owner.lamports.borrow_mut() +=amount;
        **self.vault.to_account_info().lamports.borrow_mut() -=amount;
        
        
        let vault_sate_key= self.vault.to_account_info().key();
        
                let seeds = &[VAULT.as_bytes(),vault_sate_key.as_ref(),&[self.vault.bump]];
        
                let signer_seed = &[&seeds[..]];
        
 burn_checked(
            CpiContext::new_with_signer(self.token_program.to_account_info(),BurnChecked{
                mint:self.mint.to_account_info(),
                authority:self.vault.to_account_info(),
                from:self.owner_ata.to_account_info()
            },signer_seed),
            amount,
            self.mint.decimals
            
        ).unwrap();
        
        
        
        
        
    }
}
