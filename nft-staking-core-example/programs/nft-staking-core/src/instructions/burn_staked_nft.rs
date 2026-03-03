use anchor_lang::prelude::*;
use anchor_spl::{token_interface::{Mint, TokenInterface, TokenAccount, MintToChecked, mint_to_checked}, associated_token::AssociatedToken};
use mpl_core::{
    ID as MPL_CORE_ID,
    accounts::{BaseAssetV1, BaseCollectionV1}, 
    fetch_plugin, 
    instructions::{BurnV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority}
};
use crate::state::Config;
use crate::errors::StakingError;

// Constant for time calculations
const SECONDS_PER_DAY: i64 = 86400;
const ONE_TIME_REWARD_BONUS_MUL:u32 =11; //1.1%
#[derive(Accounts)]
pub struct BurnStacked<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: PDA Update authority
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut, 
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.rewards_bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
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
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
impl<'info> BurnStacked<'info> {
    pub fn burn_stacked(&mut self, bumps: &BurnStackedBumps) -> Result<()> {
        
        // Verify NFT owner and update authority
        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;
        require!(base_asset.owner == self.user.key(), StakingError::InvalidOwner);
        require!(base_asset.update_authority == UpdateAuthority::Collection(self.collection.key()), StakingError::InvalidAuthority);
        let base_collection = BaseCollectionV1::try_from(&self.collection.to_account_info())?;
        require!(base_collection.update_authority == self.update_authority.key(), StakingError::InvalidAuthority);

        // Signer seeds for the update authority
        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        // Get current timestamp
        let current_timestamp = Clock::get()?.unix_timestamp;

        // Check if the NFT has the attribute plugin already added - return error if not
        let fetched_attribute_list = match fetch_plugin::<BaseAssetV1, Attributes>(&self.nft.to_account_info(), PluginType::Attributes) {
            Err(_) => {
                return Err(StakingError::NotStaked.into());
            }
            Ok((_, attributes, _)) => attributes,
        };

        // Extract and validate staking attributes
        let mut attribute_list: Vec<Attribute> = Vec::with_capacity(fetched_attribute_list.attribute_list.len());
        let mut staked_value: Option<&str> = None;
        let mut staked_at_value: Option<&str> = None;
        
        for attribute in &fetched_attribute_list.attribute_list {
            match attribute.key.as_str() {
                "staked" => {
                    staked_value = Some(&attribute.value);
                    attribute_list.push(Attribute { 
                        key: "staked".to_string(), 
                        value: "false".to_string() 
                    });
                }
                "staked_at" => {
                    staked_at_value = Some(&attribute.value);
                    attribute_list.push(Attribute { 
                        key: "staked_at".to_string(), 
                        value: "0".to_string() 
                    });
                }
                _ => {
                    attribute_list.push(attribute.clone());
                }
            }
        }

        require!(staked_value == Some("true"), StakingError::NotStaked);
        
        let staked_at_timestamp = staked_at_value
            .ok_or(StakingError::InvalidTimestamp)?
            .parse::<i64>()
            .map_err(|_| StakingError::InvalidTimestamp)?;

        // Calculate staked time in days
        let elapsed_seconds = current_timestamp
            .checked_sub(staked_at_timestamp)
            .ok_or(StakingError::InvalidTimestamp)?;
        
        let staked_time_days = elapsed_seconds
            .checked_div(SECONDS_PER_DAY)
            .ok_or(StakingError::InvalidTimestamp)?;

        require!(staked_time_days > 0, StakingError::FreezePeriodNotElapsed);
        require!(staked_time_days >= self.config.freeze_period as i64, StakingError::FreezePeriodNotElapsed);
        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(&[signer_seeds])?;
        
        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
             .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.user.to_account_info()))
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke()?;        
        // Calculate rewards to the user
        let bonus_points_per_stake = self.config.points_per_stake.checked_mul(ONE_TIME_REWARD_BONUS_MUL).ok_or(StakingError::Overflow)?.checked_div(10).ok_or(StakingError::Overflow)?;
        let amount = (staked_time_days as u64)
            .checked_mul(bonus_points_per_stake as u64)
            .ok_or(StakingError::Overflow)?;

        // Prepare signer seeds for config PDA
        let config_seeds = &[
            b"config",
            collection_key.as_ref(),
            &[self.config.config_bump],
        ];
        let config_signer_seeds = &[&config_seeds[..]];
        
        // Mint rewards tokens to user's ATA
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintToChecked {
            mint: self.rewards_mint.to_account_info(),
            to: self.user_rewards_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, config_signer_seeds);
        mint_to_checked(cpi_ctx, amount, self.rewards_mint.decimals)?;
        
        Ok(())
    }
}