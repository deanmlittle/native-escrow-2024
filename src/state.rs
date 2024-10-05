use bytemuck::{Pod, Zeroable};
use solana_program::{
    program::{invoke, invoke_signed},
    system_instruction::create_account,
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
    rent::Rent,
};
use spl_token::instruction::{transfer_checked, close_account};
use crate::utils::{check_eq_program_derived_address, check_eq_program_derived_address_and_get_bump};

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
    pub fn get_data_and_bump(
        maker: &Pubkey,
        escrow: &AccountInfo,
    ) -> Result<(Escrow, u8), ProgramError>  {
        // Get escrow data
        let escrow_data: Escrow = *bytemuck::try_from_bytes::<Escrow>(*escrow.data.borrow())
            .map_err(|_| ProgramError::InvalidAccountData)?;

        // Check PDA of escrow and get bump
        let bump = check_eq_program_derived_address_and_get_bump(&[b"escrow", maker.as_ref(), escrow_data.seed.to_le_bytes().as_ref()], &crate::ID, escrow.key)?;

        Ok((escrow_data, bump))
    }

    #[inline]
    pub fn init<'a>(
        seed: u64,
        receive: u64,
        mint_a: Pubkey,
        mint_b: Pubkey,
        maker: &AccountInfo<'a>,
        escrow: &AccountInfo<'a>,
    ) -> ProgramResult {
        // Check PDA of escrow and get bump
        let bump = check_eq_program_derived_address_and_get_bump(&[b"escrow", maker.key.as_ref(), seed.to_le_bytes().as_ref()], &crate::ID, escrow.key)?;

        let space = core::mem::size_of::<Escrow>();
        let rent = Rent::get()?.minimum_balance(space);

        // Create the Escrow Account
        invoke_signed(
            &create_account(
                maker.key,
                escrow.key,
                rent,
                space as u64,
                &crate::ID,
            ),
            &[
                maker.clone(), 
                escrow.clone()
            ],
            &[
                &[
                    b"escrow",
                    maker.key.as_ref(),
                    seed.to_le_bytes().as_ref(),
                    &[bump],
                ]
            ],
        )?;

        escrow.assign(&crate::ID);
        // Create the escrow
        let mut escrow_data: Escrow =
            *bytemuck::try_from_bytes_mut::<Escrow>(*escrow.data.borrow_mut())
                .map_err(|_| ProgramError::InvalidAccountData)?;
        escrow_data.clone_from(&Escrow {
            seed,
            maker: *maker.key,
            mint_a,
            mint_b,
            receive,
        });

        Ok(())
    }

    #[inline]
    pub fn deposit<'a>(
        escrow_address: &Pubkey,
        token_program: &Pubkey,
        amount: u64,
        maker_ta_a: &AccountInfo<'a>,
        mint_a: &AccountInfo<'a>,
        vault: &AccountInfo<'a>,
        maker: &AccountInfo<'a>,
    ) -> ProgramResult {
        // Check PDA of vault
        check_eq_program_derived_address(&[b"vault", escrow_address.as_ref()], &crate::ID, vault.key)?;

        // By checking this, we know our token accounts are correct by virtue of Token Program checking them
        assert!([&spl_token::ID, &spl_token_2022::ID].contains(&token_program));

        // Check if the vault is owned by the escrow
        assert_eq!(
            *escrow_address,
            *<spl_token::state::Account as spl_token::state::GenericTokenAccount>::unpack_account_owner(*vault.try_borrow_data()?)
            .ok_or(ProgramError::InvalidAccountData)?
        );
    
        // Get token decimals
        let decimals = spl_token::state::Mint::unpack(&mint_a.try_borrow_data()?)?.decimals;
    
        // Transfer the funds from the maker's token account to the vault
        invoke(
            &transfer_checked(
                token_program,
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
        )?;

        Ok(())
    }

    #[inline]
    pub fn take<'a>(
        escrow_data: Escrow,
        bump: u8,
        token_program: &Pubkey,
        mint_a: &AccountInfo<'a>,
        mint_b: &AccountInfo<'a>,
        maker: &AccountInfo<'a>,
        taker: &AccountInfo<'a>,
        escrow: &AccountInfo<'a>,
        vault: &AccountInfo<'a>,
        maker_ta_b: &AccountInfo<'a>,
        taker_ta_a: &AccountInfo<'a>,
        taker_ta_b: &AccountInfo<'a>,
    ) -> ProgramResult {

        // Check PDA of vault
        check_eq_program_derived_address(&[b"vault", escrow.key.as_ref()], &crate::ID, vault.key)?;

        // Check mints match
        assert_eq!(mint_a.key, &escrow_data.mint_a);
        assert_eq!(mint_b.key, &escrow_data.mint_b);

        // Get token decimals
        let decimals_a = spl_token::state::Mint::unpack(&mint_a.try_borrow_data()?)?.decimals;
        let decimals_b = spl_token::state::Mint::unpack(&mint_b.try_borrow_data()?)?.decimals;

        // Get token amount
        let amount = spl_token::state::Account::unpack(&vault.try_borrow_data()?)?.amount;

        // By checking this, we know our token accounts are correct by virtue of Token Program checking them
        assert!([&spl_token::ID, &spl_token_2022::ID].contains(&token_program));

         // Claim token A to taker
        invoke_signed(
            &transfer_checked(
                token_program,
                vault.key,
                mint_a.key,
                taker_ta_a.key,
                escrow.key,
                &[],
                amount,
                decimals_a,
            )?,
            &[
                vault.clone(),
                mint_a.clone(),
                taker_ta_a.clone(),
                escrow.clone(),
            ],
            &[&[
                b"escrow",
                maker.key.as_ref(),
                escrow_data.seed.to_le_bytes().as_ref(),
                &[bump],
            ]],
        )?;

        // Transfer token B to maker
        invoke(
            &transfer_checked(
                token_program,
                taker_ta_b.key,
                mint_b.key,
                maker_ta_b.key,
                taker.key,
                &[],
                escrow_data.receive,
                decimals_b,
            )?,
            &[
                taker_ta_b.clone(),
                mint_b.clone(),
                maker_ta_b.clone(),
                taker.clone(),
            ],
        )?;

        // Close the vault
        invoke_signed(
            &close_account(
                token_program, 
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
            &[&[
                b"escrow",
                maker.key.as_ref(),
                escrow_data.seed.to_le_bytes().as_ref(),
                &[bump],
            ]],
        )?;

        // Close the escrow
        let balance = escrow.lamports();
        escrow.realloc(0, false)?;
        **escrow.lamports.borrow_mut() = 0;
        **maker.lamports.borrow_mut() += balance;
        escrow.assign(&Pubkey::default());

        Ok(())
    }

    #[inline]
    pub fn refund<'a>(
        escrow_data: Escrow,
        bump: u8,
        token_program: &Pubkey,
        mint_a: &AccountInfo<'a>,
        maker: &AccountInfo<'a>,
        escrow: &AccountInfo<'a>,
        vault: &AccountInfo<'a>,
        maker_ta_a: &AccountInfo<'a>,
    ) -> ProgramResult {

        // Check PDA of vault
        check_eq_program_derived_address(&[b"vault", escrow.key.as_ref()], &crate::ID, vault.key)?;

        // Check mints match
        assert_eq!(mint_a.key, &escrow_data.mint_a);

        // Get token decimals
        let decimals = spl_token::state::Mint::unpack(&mint_a.try_borrow_data()?)?.decimals;

        // Get token amount
        let amount = spl_token::state::Account::unpack(&vault.try_borrow_data()?)?.amount;

        // By checking this, we know our token accounts are correct by virtue of Token Program checking them
        assert!([&spl_token::ID, &spl_token_2022::ID].contains(&token_program));

        // Refund the vault funds
        invoke_signed(
            &transfer_checked(
                token_program,
                vault.key,
                mint_a.key,
                maker_ta_a.key,
                escrow.key,
                &[],
                amount,
                decimals,
            )?,
            &[
                vault.clone(),
                mint_a.clone(),
                maker_ta_a.clone(),
                escrow.clone(),
            ],
            &[&[
                b"escrow",
                maker.key.as_ref(),
                escrow_data.seed.to_le_bytes().as_ref(),
                &[bump],
            ]],
        )?;

        // Close the vault
        invoke_signed(
            &close_account(
                token_program, 
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
            &[&[
                b"escrow",
                maker.key.as_ref(),
                escrow_data.seed.to_le_bytes().as_ref(),
                &[bump],
            ]],
        )?;

        // Close the escrow
        let balance = escrow.lamports();
        escrow.realloc(0, false)?;
        **escrow.lamports.borrow_mut() = 0;
        **maker.lamports.borrow_mut() += balance;
        escrow.assign(&Pubkey::default());

        Ok(())
    }
}
