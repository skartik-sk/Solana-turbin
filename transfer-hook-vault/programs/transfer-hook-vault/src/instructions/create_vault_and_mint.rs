use anchor_lang::{
    prelude::*,
    solana_program::program::invoke,
    system_program::{ },
};

use anchor_spl::{
    token_interface::{TokenInterface, TransferFeeInitialize, spl_pod::optional_keys::OptionalNonZeroPubkey, transfer_fee_initialize},
    *,
};

use solana_system_interface::instruction::create_account;
use spl_token_2022::{
    extension::*,
    instruction::{initialize_mint2, initialize_permanent_delegate},
    state::Mint,
    *,
};

use spl_token_metadata_interface::{state::TokenMetadata, *};
use spl_type_length_value::variable_len_pack::VariableLenPack;

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
        name: String,
        symbol: String,
        uri: String,
        decimal: u8,
    ) -> Result<()> {
        let extension_types = vec![
            ExtensionType::TransferHook,
            ExtensionType::TransferFeeAmount,
            ExtensionType::PermanentDelegate,
        ];
        let space = ExtensionType::try_calculate_account_len::<Mint>(&extension_types).unwrap();

        let token_metadata = TokenMetadata {
            name: name,
            symbol: symbol,
            uri: uri,
            mint: self.mint.key(),
            update_authority: OptionalNonZeroPubkey(self.admin.key()),
            additional_metadata: vec![],
        };

        let metadata_space = token_metadata.get_packed_len().unwrap() + 8;

        let lamport = Rent::get().unwrap().minimum_balance(space + metadata_space);

        let create_ix = create_account(
            &self.admin.key(),
            &self.mint.key(),
            lamport,
            space as u64,
            &self.token_program.key(),
        );

        invoke(
            &create_ix,
            &[
                self.admin.to_account_info(),
                self.mint.to_account_info(),
                self.system_program.to_account_info(),
            ],
        ).unwrap();

        msg!("Mint account created");
        msg!("logs {}",crate::ID);
        let init_hook_ix = transfer_hook::instruction::initialize(
            &self.token_program.key(),
            &self.mint.key(),
            Some(self.admin.key()),
            Some(crate::ID),
        )?;
        invoke(
            &init_hook_ix,
            &[
                self.mint.to_account_info(),
            ],
        )?;
        msg!("transfer hook added ");
        
        // transfer_fee_initialize(
        //         CpiContext::new(
        //             self.token_program.to_account_info(),
        //             TransferFeeInitialize {
        //                 token_program_id: self.token_program.to_account_info(),
        //                 mint: self.mint.to_account_info(),
        //             },
        //         ),
        //         Some(&self.admin.key()), // transfer fee config authority (update fee)
        //         Some(&self.admin.key()), // withdraw authority (withdraw fees)
        //         fee.into(),       // transfer fee basis points (% fee per transfer)
        //         decimal.into(),                     // maximum fee (maximum units of token per transfer)
        //     )?;
        // 
        // 
        let init_tran_fee_ix = transfer_fee::instruction::initialize_transfer_fee_config(
            &self.token_program.key(),
            &self.mint.key(),
            Some(&self.admin.key()),
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
        .unwrap();
        invoke(&init_perm_deli_ix, &[self.mint.to_account_info()]).unwrap();

        let init_mint_ix = initialize_mint2(
            &self.token_program.key(),
            &self.mint.key(),
            &self.vault.key(),
            Some(&self.vault.key()),
            decimal,
        )
        .unwrap();
        invoke(&init_mint_ix, &[self.mint.to_account_info()]).unwrap();
        Ok(())
    }
}
