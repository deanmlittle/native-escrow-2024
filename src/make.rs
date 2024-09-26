use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::Sysvar,
};
use spl_token::instruction::transfer_checked;

use crate::{Escrow, Make};

/// Deposit funds into vault with deterministic address derived from Signer's pubkey
pub fn process(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let Make {
        seed,
        amount,
        receive,
    } = bytemuck::try_pod_read_unaligned::<Make>(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let [maker, mint_a, mint_b, maker_ta_a, escrow, vault, token_program, _system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check PDA of escrow
    let (escrow_address, bump) = Pubkey::try_find_program_address(
        &[b"escrow", maker.key.as_ref(), seed.to_le_bytes().as_ref()],
        &crate::ID,
    )
    .ok_or(ProgramError::InvalidAccountData)?;
    assert_eq!(escrow_address, *escrow.key);

    // Check PDA of vault
    let vault_address =
        Pubkey::try_find_program_address(&[b"vault", escrow_address.as_ref()], &crate::ID)
            .ok_or(ProgramError::InvalidAccountData)?
            .0;
    assert_eq!(vault_address, *vault.key);

    // Init escrow account
    let space = core::mem::size_of::<Escrow>();
    let rent = Rent::get()?.minimum_balance(space);

    // Initialize Escrow account
    invoke_signed(
        &create_account(
            maker.key,
            escrow.key,
            rent,
            core::mem::size_of::<Escrow>() as u64,
            &crate::ID,
        ),
        &[maker.clone(), escrow.clone()],
        &[&[
            b"escrow",
            maker.key.as_ref(),
            seed.to_le_bytes().as_ref(),
            &[bump],
        ]],
    )?;

    // Initialize escrow data
    Escrow::init(seed, receive, *maker.key, *mint_a.key, *mint_b.key, escrow)?;

    // By checking this, we know our token accounts are correct by virtue of Token Program checking them
    assert!([&spl_token::ID, &spl_token_2022::ID].contains(&token_program.key));

    // Make sure escrow owns the vault
    assert_eq!(
        escrow_address,
        *<spl_token::state::Account as spl_token::state::GenericTokenAccount>::unpack_account_owner(*vault.try_borrow_data()?)
        .ok_or(ProgramError::InvalidAccountData)?
    );

    // Get token decimals
    let decimals = spl_token::state::Mint::unpack(&mint_a.try_borrow_data()?)?.decimals;

    invoke(
        &transfer_checked(
            token_program.key,
            maker_ta_a.key,
            mint_a.key,
            vault.key,
            maker.key,
            &[],
            amount,
            decimals,
        )?,
        &[
            maker_ta_a.clone(),
            mint_a.clone(),
            vault.clone(),
            maker.clone(),
        ],
    )
}
