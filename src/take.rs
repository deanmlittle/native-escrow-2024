use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
};
use crate::Escrow;

/// Taker pays funds to Maker and claims funds in Vault
pub fn process(accounts: &[AccountInfo<'_>]) -> ProgramResult {
    let [taker, maker, mint_a, mint_b, taker_ta_a, taker_ta_b, maker_ta_b, escrow, vault, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    // Check & Get escrow account data and bump
    let (escrow_data, bump) = Escrow::get_data_and_bump(maker.key, escrow)?;

    // Take: Claim token A to taker, Transfer token B to maker, Close the vault & escrow
    Escrow::take(escrow_data, bump, token_program.key, mint_a, mint_b, maker, taker, escrow, vault, maker_ta_b, taker_ta_a, taker_ta_b)
}
