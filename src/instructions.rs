use bytemuck::{Pod, Zeroable};
use solana_program::program_error::ProgramError;
pub enum EscrowInstructions {
    Make,
    Take,
    Refund,
}

impl TryFrom<&u8> for EscrowInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Make),
            1 => Ok(Self::Take),
            2 => Ok(Self::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct Make {
    pub seed: u64,
    pub amount: u64,
    pub receive: u64,
}

impl TryFrom<&[u8]> for Make {
    
    type Error = ProgramError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        bytemuck::try_pod_read_unaligned::<Self>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}
