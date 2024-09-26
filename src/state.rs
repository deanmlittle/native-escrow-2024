use bytemuck::{Pod, Zeroable};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
}

impl Escrow {
    #[inline]
    pub fn init(seed: u64, receive: u64, maker: Pubkey, mint_a: Pubkey, mint_b: Pubkey, escrow: &AccountInfo<'_>) -> ProgramResult {
        escrow.assign(&crate::ID);
        // Create the escrow
        let mut escrow_data: Escrow = *bytemuck::try_from_bytes_mut::<Escrow>(*escrow.data.borrow_mut()).map_err(|_| ProgramError::InvalidAccountData)?;
        escrow_data.clone_from(&Escrow {
            seed,
            maker,
            mint_a,
            mint_b,
            receive
        });
        Ok(())
    }
}