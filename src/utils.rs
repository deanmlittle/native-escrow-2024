use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[inline]
pub fn check_eq_program_derived_address(
    seeds: &[&[u8]],
    program_id: &Pubkey,
    address: &Pubkey,
) -> Result<(), ProgramError> {
    let (derived_address, _) = Pubkey::try_find_program_address(seeds, program_id).ok_or(ProgramError::InvalidAccountData)?;
    Ok(assert!(derived_address.eq(address)))
}

#[inline]
pub fn check_eq_program_derived_address_and_get_bump(
    seeds: &[&[u8]],
    program_id: &Pubkey,
    address: &Pubkey,
) -> Result<u8, ProgramError> {
    let (derived_address, bump) = Pubkey::try_find_program_address(seeds, program_id).ok_or(ProgramError::InvalidAccountData)?;
    assert!(derived_address.eq(address));
    Ok(bump)
}