use anchor_lang::prelude::*;

mod state;
mod instructions;
mod errors;
use instructions::*;
mod utils;
declare_id!("HsptKegpGaqCjvrcbAQspZSHNPt8cs49ehbyYVXLur8J");

#[program]
pub mod nft_staking_core {
    use super::*;

    pub fn create_collection(ctx: Context<CreateCollection>, name: String, uri: String) -> Result<()> {
        ctx.accounts.create_collection(name, uri, &ctx.bumps)
    }

    pub fn mint_nft(ctx: Context<Mint>, name: String, uri: String) -> Result<()> {
        ctx.accounts.mint_nft(name, uri, &ctx.bumps)
    }

    pub fn initialize_config(ctx: Context<InitConfig>, points_per_stake: u32, freeze_period: u8) -> Result<()> {
        ctx.accounts.init_config(points_per_stake, freeze_period, &ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(&ctx.bumps)
    }
    
    pub fn claim_rewards(ctx:Context<ClaimRewards>)->Result<()>{
        ctx.accounts.claim_rewards(&ctx.bumps)
    }
    
    pub fn burn_staked_nft(ctx:Context<BurnStacked>)->Result<()>{
        ctx.accounts.burn_stacked(&ctx.bumps)
    }
    pub fn transfer(ctx:Context<Transfer>)->Result<()>{
        ctx.accounts.transfer()
    }

    pub fn update_oracle(ctx:Context<UpdateOracle>)->Result<()>{
        ctx.accounts.update_oracle()
    }
}
