use crate::{errors::StakingError, state::Validation};
use crate::state::Config;
use anchor_lang::prelude::*;
use mpl_core::instructions::TransferV1CpiBuilder;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        AddCollectionPluginV1CpiBuilder, AddPluginV1CpiBuilder,
        UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder,
    },
    types::{
        Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority,
    },
    ID as MPL_CORE_ID,
};

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    

    /// CHECK: new owner
    pub next_owner: UncheckedAccount<'info>,
    
    
    
    #[account(
mut,
        seeds = [b"oracle"],
        bump = oracle.bump
    )]
    pub oracle: Account<'info, Validation>,
    /// CHECK: NFT account will be checked by the mpl core program
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> Transfer<'info> {
    pub fn transfer(&mut self, bumps: &TransferBumps) -> Result<()> {
//just send nft 


TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
    .asset(&self.nft.to_account_info())
    .collection(Some(&self.collection.to_account_info()))
    .payer(&self.user.to_account_info())
    .new_owner(&self.next_owner.to_account_info())
    .authority(Some(&self.user.to_account_info()))
    .add_remaining_account(&self.oracle.to_account_info(), true, false)
    .system_program(Some(&self.system_program.to_account_info()))
    .invoke()?;
        Ok(())
    }
}
