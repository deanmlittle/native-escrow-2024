use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
};
use crate::{Escrow, Make};

/// Deposit funds into vault derived from Makers's pubkey and seed
pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Make {
        seed,
        amount,
        receive,
    } = Make::try_from(data)?;

    let [maker, mint_a, mint_b, maker_ta_a, escrow, vault, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Initialize escrow account & data
    Escrow::init(seed, receive, *mint_a.key, *mint_b.key, &maker, &escrow)?;

    // Deposit funds into vault
    Escrow::deposit(escrow.key, token_program.key, amount, &maker_ta_a, &mint_a, &vault, &maker)
}
