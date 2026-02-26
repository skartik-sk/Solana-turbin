pub mod make;
pub mod take;
pub mod cancel;
pub mod makev2;
pub mod takev2;

pub use make::*;
pub use take::*;
pub use cancel::*;
pub use makev2::*;
pub use takev2::*;
use pinocchio::error::ProgramError;

pub enum EscrowInstrctions {
    Make = 0,
    Take = 1,
    Cancel = 2,
    MakeV2 = 3,
    TakeV2 = 4,
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
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}