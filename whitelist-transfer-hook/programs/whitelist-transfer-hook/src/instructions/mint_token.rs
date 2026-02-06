use anchor_lang::{prelude::{program::invoke, *}, system_program::{CreateAccount, create_account}};
use anchor_spl::{token_2022::{Token2022,  spl_token_2022::{
       extension::{ExtensionType, transfer_hook::instruction::initialize as init_transfer_hook},
       instruction::initialize_mint2,
       state::Mint as Token2022Mint,
   }}, token_interface::{
    Mint, 
    TokenInterface,
}};

use crate::state::Whitelist;

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    
    /// CHECK: will be initialize manually 
    #[account(
        mut ,signer
    )]
    pub mint: AccountInfo<'info>,
    /// CHECK: ExtraAccountMetaList Account, will be checked by the transfer hook
    #[account(mut)]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self, bumps: &TokenFactoryBumps,decimals:u8) -> Result<()> {
        
                let extension_types = vec![ExtensionType::TransferHook];
                let space = ExtensionType::try_calculate_account_len::<Token2022Mint>(&extension_types)
                    .map_err(|_| ProgramError::AccountDataTooSmall)?;
        
                msg!("Mint account space needed: {} bytes", space);
        
                // Calculate rent
                let lamports = Rent::get()?.minimum_balance(space);
        
                // Create the mint account via CPI to System Program
                create_account(
                    CpiContext::new(
                        self.system_program.to_account_info(),
                        CreateAccount {
                            from: self.user.to_account_info(),
                            to: self.mint.to_account_info(),
                        },
                    ),
                    lamports,
                    space as u64,
                    &self.token_program.key(),
                )?;
        
                msg!("Mint account created");
        
                // Initialize the TransferHook extension via CPI
                let init_hook_ix = init_transfer_hook(
                    &self.token_program.key(),
                    &self.mint.key(),
                    Some(self.user.key()),
                    Some(crate::ID),
                )?;
        
                invoke(&init_hook_ix, &[self.mint.to_account_info()])?;
        
                msg!("Transfer hook extension initialized");
        
                // Initialize the base mint via CPI
                let init_mint_ix = initialize_mint2(
                    &self.token_program.key(),
                    &self.mint.key(),
                    &self.user.key(),
                    Some(&self.user.key()),
                    decimals,
                )?;
        
                invoke(&init_mint_ix, &[self.mint.to_account_info()])?;
        


        Ok(())
    }
}