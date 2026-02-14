#![allow(unexpected_cfgs, deprecated)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("53GFYSJPbrYcaqD3o54z5WCWcCM8WGqixgUjc4nsw2tY");

#[program]
pub mod tuktuk_get_oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.init_llm(&ctx.bumps)
    }

    pub fn analyze_user(
        ctx: Context<AnalyzeUser>,
        user_pubkey: Pubkey,
        user_data: String,
    ) -> Result<()> {
        ctx.accounts.analyse_user(user_pubkey, user_data)
    }

    pub fn callback_from_agent(ctx: Context<CallbackFromAgent>, response: String) -> Result<()> {
        ctx.accounts.callback_from_agent(response, &ctx.bumps)
    }

    // this ix is to only forward the result from the llm to the frontend
    pub fn get_analysis(ctx: Context<GetAnalysis>) -> Result<String> {
        ctx.accounts.get_analysis()
    }
}