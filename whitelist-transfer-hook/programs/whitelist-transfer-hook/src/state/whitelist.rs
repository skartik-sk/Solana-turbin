use anchor_lang::prelude::*;

use crate::constant::ANCHOR_DISCRIMINATOR_SIZE;

#[account]
pub struct Whitelist {
    pub address: Pubkey,
    pub bump: u8,
}

impl Whitelist {
    pub const SIZE: usize = ANCHOR_DISCRIMINATOR_SIZE + 32 + 1;
}