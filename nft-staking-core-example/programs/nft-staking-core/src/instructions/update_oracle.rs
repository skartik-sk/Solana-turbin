use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

// use mpl_core::types::{ExternalValidationResult, OracleValidation};

use crate::{errors::StakingError, state::{ExternalValidationResult, OracleValidation, Validation}, utils::{REWARD_IN_LAMPORTS, is_transferring_allowed}};


#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
mut,
        seeds = [b"oracle"],
        bump = oracle.bump
    )]
    pub oracle: Account<'info, Validation>,
    #[account(
        seeds = [b"vault_for_reward", oracle.key().as_ref()],
        bump= oracle.vault_bump,
    )]
    pub vault_for_reward: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateOracle<'info> {

pub fn update_oracle(&mut self) -> Result<()> {
    let is_allow = is_transferring_allowed(Clock::get()?.unix_timestamp);
    match  is_allow{
        true => {
            require!(
                self.oracle.validation == OracleValidation::V1 { 
                    transfer: ExternalValidationResult::Rejected,
                    create: ExternalValidationResult::Pass,
                    burn: ExternalValidationResult::Pass,
                    update: ExternalValidationResult::Pass
                },
                StakingError::AlreadyUpdated
            );
            self.oracle.validation = OracleValidation::V1 { 
                transfer: ExternalValidationResult::Approved,
                create: ExternalValidationResult::Pass,
                burn: ExternalValidationResult::Pass,
                update: ExternalValidationResult::Pass
            };
        }
        false => {
            require!(
                self.oracle.validation == OracleValidation::V1 {
                    transfer: ExternalValidationResult::Approved,
                    create: ExternalValidationResult::Pass,
                    burn: ExternalValidationResult::Pass,
                    update: ExternalValidationResult::Pass
                },
                StakingError::AlreadyUpdated
            );
            self.oracle.validation =OracleValidation::V1 {
                transfer: ExternalValidationResult::Rejected,
                create: ExternalValidationResult::Pass,
                burn: ExternalValidationResult::Pass,
                update: ExternalValidationResult::Pass,
            };
        }
    }
    let vault_for_reward_lamports = self.vault_for_reward.lamports();
    let oracle_key = self.oracle.key().clone();
    let signer_seeds = &[b"vault_for_reward", oracle_key.as_ref(), &[self.oracle.vault_bump]];
    
  
        // Reward cranker for updating Oracle within 15 minutes of market open or close
if is_allow &&  vault_for_reward_lamports > REWARD_IN_LAMPORTS{
    

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(), 
                Transfer {
                    from: self.vault_for_reward.to_account_info(),
                    to: self.signer.to_account_info(),
                }, 
                &[signer_seeds]
            ),
            REWARD_IN_LAMPORTS
        )?;
}
    Ok(())
    }


}