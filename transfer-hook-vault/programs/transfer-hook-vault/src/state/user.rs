use anchor_lang::prelude::*;


#[account]
pub struct User {
    pub address:Pubkey,
    pub bump:u8
}
    
impl User {
    pub const  LEN :usize  = 32+1;
}
