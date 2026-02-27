pub mod make;
pub mod take;
pub mod cancel;
pub mod wincode;
pub mod serde;
pub mod bytemuck;
pub mod borsh;

pub use make::*;
pub use take::*;
pub use cancel::*;
pub use wincode::*;
pub use serde::*;
pub use bytemuck::*;
pub use borsh::*;

use pinocchio::error::ProgramError;

pub enum EscrowInstrctions {
    Make = 0,
    Take = 1,
    Cancel = 2,
    MakeV2 = 3,
    TakeV2 = 4,
    MakeV3 = 7,
    TakeV3 = 8,
    MakeV4 = 9,
    TakeV4 = 10,
    MakeV5 = 5,
    TakeV5 = 6,
}

impl TryFrom<&u8> for EscrowInstrctions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowInstrctions::Make),
            1 => Ok(EscrowInstrctions::Take),
            2 => Ok(EscrowInstrctions::Cancel),
            3 => Ok(EscrowInstrctions::MakeV2),
            4 => Ok(EscrowInstrctions::TakeV2),
            5 => Ok(EscrowInstrctions::MakeV5),
            6 => Ok(EscrowInstrctions::TakeV5),
            7 => Ok(EscrowInstrctions::MakeV3),
            8 => Ok(EscrowInstrctions::TakeV3),
            9 => Ok(EscrowInstrctions::MakeV4),
            10 => Ok(EscrowInstrctions::TakeV4),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}