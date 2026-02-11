#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod state;
mod instructions;

use instructions::*;

declare_id!("3o8P79bz6y2QKXJTgrMdviA47GN28qgp5vYsFv8J77cC");

#[ephemeral]
#[program]
pub mod er_state_account {

    use super::*;

    pub fn initialize(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;
        
        Ok(())
    }

    pub fn update(ctx: Context<UpdateUser>, new_data: u64) -> Result<()> {
        ctx.accounts.update(new_data)?;
        
        Ok(())
    }

    pub fn update_commit(ctx: Context<UpdateCommit>, new_data: u64) -> Result<()> {
        ctx.accounts.update_commit(new_data)?;
        
        Ok(())
    }

    pub fn delegate(ctx: Context<Delegate>) -> Result<()> {
        ctx.accounts.delegate()?;
        
        Ok(())
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        ctx.accounts.undelegate()?;
        
        Ok(())
    }

    pub fn close(ctx: Context<CloseUser>) -> Result<()> {
        ctx.accounts.close()?;
        
        Ok(())
    }
    
    pub fn request_for_random(ctx: Context<DoUpdateWithVRFCtx>,client_seed: u8) -> Result<()> {
        ctx.accounts.request_for_random(client_seed)?;
        
        Ok(())
    }
    
    pub fn get_random(ctx: Context<CallbackUpdateWithVRFCtx>, randomness: [u8; 32],) -> Result<()> {
        ctx.accounts.callback_get_random(randomness)?;
        
        Ok(())
    }
    
    
}

