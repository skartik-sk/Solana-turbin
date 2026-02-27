#![allow(unexpected_cfgs)]
use pinocchio::{AccountView, entrypoint, Address, ProgramResult, address::declare_id, error::ProgramError};
use pinocchio_pubkey::derive_address

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

    // match EscrowInstrctions::try_from(discriminator)? {
    //     EscrowInstrctions::Make => instructions::process_make_instruction(accounts, data)?,
    //     EscrowInstrctions::Take => instructions::process_take_instruction(accounts, data)?,
    //     EscrowInstrctions::Cancel => instructions::process_cancel_instruction(accounts, data)?,
    //     EscrowInstrctions::MakeV2 => instructions::process_makev2_instruction(accounts, data)?,
    //     EscrowInstrctions::TakeV2 => instructions::process_takev2_instruction(accounts, data)?,
    //     EscrowInstrctions::MakeV3 => instructions::process_makev3_instruction(accounts, data)?,
    //     EscrowInstrctions::TakeV3 => instructions::process_takev3_instruction(accounts, data)?,
    //     EscrowInstrctions::MakeV4 => instructions::process_makev4_instruction(accounts, data)?,
    //     EscrowInstrctions::TakeV4 => instructions::process_takev4_instruction(accounts, data)?,
    //     EscrowInstrctions::MakeV5 => instructions::process_makev5_instruction(accounts, data)?,
    //     EscrowInstrctions::TakeV5 => instructions::process_takev5_instruction(accounts, data)?,
    //     _ => return Err(ProgramError::InvalidInstructionData),
    // }
    Ok(())
}