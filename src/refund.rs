use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
};
use crate::Escrow;

/// Refund funds in vault to Maker's token account
pub fn process(accounts: &[AccountInfo<'_>]) -> ProgramResult {
    let [maker, mint_a, maker_ta_a, escrow, vault, token_program, _system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Make sure the maker is a signer
    assert!(maker.is_signer);

    // Check & Get escrow account data and bump
    let (escrow_data, bump) = Escrow::get_data_and_bump(maker.key, escrow)?;

    // Refund: Transfer token A from vault to maker, Close the vault & escrow
    Escrow::refund(escrow_data, bump, token_program.key, mint_a, maker, escrow, vault, maker_ta_a)
}
