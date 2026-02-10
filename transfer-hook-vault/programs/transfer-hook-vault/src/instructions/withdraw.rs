use anchor_lang::{prelude::*, solana_program::{program::invoke_signed, system_instruction}, system_program::{Transfer as SolTransfer, transfer}};
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
        mut
    )]
    pub mint: InterfaceAccount<'a,Mint>,

    #[account(
        mut,
     seeds=[VAULT.as_bytes(),vault.admin.key().as_ref()],
     bump,
 )]
    pub vault: Account<'a, Vault>,

    #[account(seeds=[USER.as_bytes(),owner.key().as_ref()], bump)]
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
    pub fn withdraw(&mut self, amount: u64)->Result<()> {
               require!(
                   self.user.address.key().eq(&self.owner.key()),
                   MyError::Unauthorized
               );
               
               
               let vault_minimum_balance = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());
               
               
               let vault_balance = self.vault.to_account_info().lamports();
               require!(
                   vault_balance >= amount + vault_minimum_balance,
                   MyError::InsufficientFunds
               );
               
              
           
        
        
        
        let vault_sate_key= self.vault.admin.key();
        
                let seeds = &[VAULT.as_bytes(),vault_sate_key.as_ref(),&[self.vault.bump]];
        
                let signer_seed = &[&seeds[..]];
                
               

                
                
                
                // let transfer_ix = system_instruction::transfer(
                //             &self.vault.key(),
                //             &self.owner.key(),
                //             amount,
                //         );
                        
                //         invoke_signed(
                //             &transfer_ix,
                //             &[
                //                 self.vault.to_account_info(),
                //                 self.owner.to_account_info(),
                //                 self.system_program.to_account_info(),
                //             ],
                //             signer_seed,
                //         )?;
        
 burn_checked(
            CpiContext::new_with_signer(self.token_program.to_account_info(),BurnChecked{
                mint:self.mint.to_account_info(),
                authority:self.vault.to_account_info(),
                from:self.owner_ata.to_account_info()
            },signer_seed),
            amount,
            self.mint.decimals
            
        )?;
 
 **self.vault.to_account_info().try_borrow_mut_lamports()? -= amount;
 **self.owner.to_account_info().try_borrow_mut_lamports()? += amount;
 


 
 
 
 // let tx = SolTransfer{
 //             from:self.vault.to_account_info(),
 //             to:self.owner.to_account_info()
 //         };
 
         
 //         let tra = CpiContext::new_with_signer(self.system_program.to_account_info(), tx,signer_seed);
 
 //         transfer(tra, amount)?;
        
 Ok(())
        
        
    }
}
