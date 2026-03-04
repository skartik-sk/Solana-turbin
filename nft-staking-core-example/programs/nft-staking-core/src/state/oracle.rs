use anchor_lang::prelude::*;
// use mpl_core::types::OracleValidation;


#[account]
pub struct Validation {
    pub validation: OracleValidation,
    pub bump: u8,
      pub vault_bump: u8,
}
impl Validation {
    pub fn size() -> usize {
        8 // anchor discriminator
        + 5 // validation
        +2
    }
}
#[derive(AnchorDeserialize,AnchorSerialize,Clone,PartialEq)]
pub enum OracleValidation {
    Uninitialized,
    V1 {
        create: ExternalValidationResult,
        transfer: ExternalValidationResult,
        burn: ExternalValidationResult,
        update: ExternalValidationResult,
    },
}
#[derive(AnchorDeserialize,AnchorSerialize,Clone,PartialEq)]
pub enum ExternalValidationResult {
    Approved,
    Rejected,
    Pass,
}
