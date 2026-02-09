use anchor_lang::prelude::*;


#[account]
pub struct Vault {
    pub mint_token: Pubkey,
    pub admin:Pubkey,
    pub fees :u8,
    pub bump: u8,
}
    
impl Vault {
    pub const  LEN :usize  = 32+32+1+1;
}
