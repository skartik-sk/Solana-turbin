#![allow(unexpected_cfgs)]
use pinocchio::{AccountView, entrypoint, Address, ProgramResult, address::declare_id, error::ProgramError};

use crate::instruction::FundInstruction;
 mod error;
 mod constant;
 mod state;
 mod instruction;

mod tests;

entrypoint!(process_instruction);

declare_id!("9piQZir4QXTh76Xt9HwVSFtisTud8paBcWWeea6qCJxS");

pub fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {

    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data.split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundInstruction::try_from(discriminator)? {
        FundInstruction::Initialize => instruction::process_initialize_instruction(accounts, data)?,
        FundInstruction::Contribute => instruction::process_contribute_instruction(accounts, data)?,
        FundInstruction::Checker => instruction::process_checker_instruction(accounts, data)?,
        FundInstruction::Refund => instruction::process_refund_instruction(accounts, data)?,
        _ => return Err(ProgramError::InvalidInstructionData),
    }
    Ok(())
}