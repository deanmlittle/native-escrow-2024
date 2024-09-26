use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::{invoke, invoke_signed}, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey
};
use spl_token::instruction::{close_account, transfer_checked};

use crate::Escrow;

/// Deposit funds into vault with deterministic address derived from Signer's pubkey
pub fn process(accounts: &[AccountInfo<'_>]) -> ProgramResult {
    let [
        taker,
        maker, 
        mint_a,
        mint_b,
        taker_ta_a,
        taker_ta_b,
        maker_ta_b,
        escrow,
        vault,
        token_program,
        _system_program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Get escrow data
    let escrow_data: Escrow = *bytemuck::try_from_bytes::<Escrow>(*escrow.data.borrow()).map_err(|_| ProgramError::InvalidAccountData)?;

    // Check PDA of escrow
    let (escrow_address, bump)= Pubkey::try_find_program_address(&[b"escrow", maker.key.as_ref(), escrow_data.seed.to_le_bytes().as_ref()], &crate::ID).ok_or(ProgramError::InvalidAccountData)?;
    assert_eq!(escrow_address, *escrow.key);

    // Check PDA of vault
    let vault_address = Pubkey::try_find_program_address(&[b"vault", escrow_address.as_ref()], &crate::ID).ok_or(ProgramError::InvalidAccountData)?.0;
    assert_eq!(vault_address, *vault.key);

    // Check mints match
    assert_eq!(mint_a.key, &escrow_data.mint_a);
    assert_eq!(mint_b.key, &escrow_data.mint_b);

    // Get balance of vault
    let amount = spl_token::state::Account::unpack(&vault.try_borrow_data()?)?.amount;

    // Get token A decimals
    let decimals_a = spl_token::state::Mint::unpack(&mint_a.try_borrow_data()?)?.decimals;

    // Get token A decimals
    let decimals_b = spl_token::state::Mint::unpack(&mint_b.try_borrow_data()?)?.decimals;

    // Transfer token B to maker
    invoke(
        &transfer_checked(
            token_program.key, 
            taker_ta_b.key, 
            mint_b.key,
            maker_ta_b.key, 
            taker.key,
            &[],
            escrow_data.receive,
            decimals_b
        )?, 
        &[
                taker_ta_b.clone(),
                mint_b.clone(),
                maker_ta_b.clone(),
                taker.clone(),
        ]
    )?;

    // Claim token A to taker
    invoke_signed(
        &transfer_checked(
            token_program.key, 
            vault.key, 
            mint_a.key,
            taker_ta_a.key, 
            escrow.key,
            &[],
            amount,
            decimals_a
        )?, 
        &[
            vault.clone(),
            mint_a.clone(),
            taker_ta_a.clone(),
            escrow.clone(),
        ],
        &[&[b"escrow", maker.key.as_ref(), escrow_data.seed.to_le_bytes().as_ref(), &[bump]]]
    )?;

    // Close the vault
    invoke_signed(
        &close_account(
            token_program.key,
            vault.key, 
            maker.key, 
            escrow.key,
            &[]
        )?, 
        &[
            vault.clone(),
            maker.clone(),
            escrow.clone()
        ], 
        &[&[b"escrow", maker.key.as_ref(), escrow_data.seed.to_le_bytes().as_ref(), &[bump]]]
    )?;

    // Close the escrow
    let balance = escrow.lamports();
    escrow.realloc(0, false)?;
    **escrow.lamports.borrow_mut() = 0;
    **maker.lamports.borrow_mut() += balance;
    escrow.assign(&Pubkey::default());
    Ok(())
}