use anchor_lang::prelude::*;

use crate::constant::ANCHOR_DISCRIMINATOR_SIZE;


#[account]
pub struct Admin {
    pub address: Pubkey,
    pub bump: u8,
}

impl Admin {
    pub const LEN: usize = ANCHOR_DISCRIMINATOR_SIZE + 32 + 1;
}