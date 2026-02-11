use anchor_lang::prelude::*;
use ephemeral_vrf_sdk::anchor::vrf;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;

use crate::instruction::GetRandom;
use crate::state::UserAccount;
use crate::ID;
#[vrf]
#[derive(Accounts)]
pub struct DoUpdateWithVRFCtx<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    /// CHECK: The oracle queue
    #[account(mut  )]
    pub oracle_queue: AccountInfo<'info>,
}

impl<'info> DoUpdateWithVRFCtx<'info> {
    pub fn request_for_random(&mut self, client_seed: u8) -> Result<()> {
        msg!("Requesting randomness...");
        msg!("List of accounts user {}, user_account {}, oracle_queue {}", self.user.key(), self.user_account.key(), self.oracle_queue.key());
        let ix = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.user.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: ID,
            callback_discriminator: GetRandom::DISCRIMINATOR.to_vec(),
            caller_seed: [client_seed; 32],
            // Specify any account that is required by the callback
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            ..Default::default()
        });
        self.invoke_signed_vrf(&self.user.to_account_info(), &ix)?;
        Ok(())
    }
}
// Request Randomness

#[derive(Accounts)]
pub struct CallbackUpdateWithVRFCtx<'info> {
    /// This check ensure that the vrf_program_identity (which is a PDA) is a singer
    /// enforcing the callback is executed by the VRF program trough CPI
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

impl<'a> CallbackUpdateWithVRFCtx<'a> {
    // Consume Randomness
    pub fn callback_get_random(&mut self, randomness: [u8; 32]) -> Result<()> {
        msg!("List of accounts vrf_program_identity {}, user_account {}", self.vrf_program_identity.key(), self.user_account.key());
        let rnd_u8 = ephemeral_vrf_sdk::rnd::random_u8_with_range(&randomness, 1, 100);
        msg!("Consuming random number: {:?}", rnd_u8);
        self.user_account.data = rnd_u8.into(); // Update the player's last result
        Ok(())
    }
}
