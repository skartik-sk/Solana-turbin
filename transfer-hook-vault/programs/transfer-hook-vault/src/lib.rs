use anchor_lang::prelude::*;

declare_id!("63Qf2juPp7zG4cumHyT23k3v23dVaoDeqruSHWyNCy9W");
pub mod instructions;
pub mod state;
pub mod constatnt;
pub mod error;
pub use instructions::*;



#[program]
pub mod transfer_hook_vault {

    use spl_discriminator::SplDiscriminate;
    use spl_tlv_account_resolution::state::ExtraAccountMetaList;
    use spl_transfer_hook_interface::instruction::ExecuteInstruction;

    use super::*;
    

    pub fn crate_vault_and_mint(ctx: Context<CreateVault>,fee: u8, name: String, symbol: String, uri: String, decimal: u8) -> Result<()> {
        ctx.accounts.mint_token(fee, name, symbol, uri, decimal);
        ctx.accounts.create_vault(fee, ctx.bumps);
        Ok(())
    }

    pub fn add_to_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.add_to_whitelist(user,&ctx.bumps)
    }

    pub fn remove_from_whitelist(ctx: Context<WhitelistOperations>, user: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist(user)
    }
    
    pub fn deposit(ctx: Context<Deposit>, amount:u64) -> Result<()> {
        ctx.accounts.deposit(amount);
            Ok(())
          
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount:u64) -> Result<()> {
    ctx.accounts.withdraw(amount);
    Ok(())
    }
    
    pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {

        msg!("Initializing Transfer Hook...");

        // Get the extra account metas for the transfer hook
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas
        ).unwrap();

        Ok(())
    }

    #[instruction(discriminator = <spl_transfer_hook_interface::instruction::ExecuteInstruction as spl_discriminator::SplDiscriminate>::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        // Call the transfer hook logic
        msg!("trasfer-hook call");
        ctx.accounts.transfer_hook(amount)
    }
       
}

#[derive(Accounts)]
pub struct Initialize {}
