use anchor_lang::prelude::*;
use mpl_core::types::OracleValidation;


#[account]

pub struct Validation {
    pub validation: OracleValidation,
}
impl Validation {
    pub fn size() -> usize {
        8 // anchor discriminator
        + 5 // validation
    }
}
// pub enum OracleValidation {
//     Uninitialized,
//     V1 {
//         create: ExternalValidationResult,
//         transfer: ExternalValidationResult,
//         burn: ExternalValidationResult,
//         update: ExternalValidationResult,
//     },
// }
// pub enum ExternalValidationResult {
//     Approved,
//     Rejected,
//     Pass,
// }