
use anchor_lang::{prelude::*, solana_program::program::invoke, system_program::{CreateAccount, create_account}};
use anchor_spl::{token_2022::{Token2022,  spl_token_2022::{
       extension::{ExtensionType, transfer_hook::instruction::initialize as add_transfer_hook,
           transfer_fee::instruction::initialize_transfer_fee_config },
       instruction::initialize_mint2,
       state::Mint as TokenMint,
   }}, token_interface::{
    Mint, 
    TokenInterface, spl_pod::optional_keys::OptionalNonZeroPubkey,
}};


use spl_token_2022::{
    instruction::{ initialize_permanent_delegate},


};

use crate::{constatnt::VAULT, state::Vault};

#[derive(Accounts)]
pub struct CreateVault<'a> {
    #[account(mut)]
    pub admin: Signer<'a>,

    #[account(
    init,
    payer=admin,
    space=Vault::LEN+8,
    seeds=[VAULT.as_bytes(),admin.key().as_ref()],
    bump,
)]
    pub vault: Account<'a, Vault>,

    /// CHECK: We will create and initialize this account manually
    #[account(mut, signer)]
    pub mint: AccountInfo<'a>,

    pub system_program: Program<'a, System>,
    pub token_program: Interface<'a, TokenInterface>,
}

impl<'a> CreateVault<'a> {
    pub fn create_vault(&mut self, fees: u8, bump: CreateVaultBumps) -> Result<()> {
        self.vault.set_inner(Vault {
            mint_token: self.mint.key(),
            admin: self.admin.key(),
            fees,
            bump: bump.vault,
        });
        Ok(())
    }

    pub fn mint_token(
        &mut self,
        fee: u8,
        decimal: u8,
    ) -> Result<()> {
        let extension_types = vec![
            ExtensionType::TransferHook,
            ExtensionType::TransferFeeConfig,
            ExtensionType::PermanentDelegate,
        ];
        let space = ExtensionType::try_calculate_account_len::<TokenMint>(&extension_types).unwrap();

        let lamport = Rent::get().unwrap().minimum_balance(space );

        create_account(
            CpiContext::new(
                self.system_program.to_account_info(),
                CreateAccount {
                    from: self.admin.to_account_info(),
                    to: self.mint.to_account_info(),
                },
            ),
            lamport,
            space as u64,
            &self.token_program.key(),
        )?;
        
        let init_hook_ix = add_transfer_hook(
            &self.token_program.key(),
            &self.mint.key(),
            Some(self.vault.key()),
            Some(crate::ID),
        )?;

        invoke(&init_hook_ix, &[self.mint.to_account_info()])?;

        msg!("Transfer hook extension initialized");


        let init_tran_fee_ix = initialize_transfer_fee_config(
            &self.token_program.key(),
            &self.mint.key(),
            Some(&self.vault.key()),
            Some(&self.admin.key()),
            fee.into(),             // 100 = 1%
            ( decimal).into(), // max 100 token = 100sol
        )?;
        
        invoke(&init_tran_fee_ix, &[self.mint.to_account_info()])?;
        msg!("trasection fee added ");

        let init_perm_deli_ix = initialize_permanent_delegate(
            &self.token_program.key(),
            &self.mint.key(),
            &self.vault.key(),
        )
        ?;
        invoke(&init_perm_deli_ix, &[self.mint.to_account_info()])?;
 msg!("premanent deligation  added ");
 

       
        let init_mint_ix = initialize_mint2(
            &self.token_program.key(),
            &self.mint.key(),
            &self.vault.key(),
            Some(&self.vault.key()),
            decimal,
        )?;
        msg!("{:?}",init_mint_ix);

        invoke(&init_mint_ix, &[self.mint.to_account_info()])?;

         msg!("mint  added ");
        Ok(())
    }
}
