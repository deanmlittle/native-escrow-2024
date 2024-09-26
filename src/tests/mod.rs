use bytemuck::bytes_of;
use core::mem;
use mollusk_svm::{
    program::{self, program_account},
    result::ProgramResult,
    Mollusk,
};
use solana_program::instruction::AccountMeta;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    instruction::Instruction,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::AccountState;

use crate::Escrow;

#[test]
fn make() {
    // Add our built program binary
    let mut mollusk: Mollusk = Mollusk::new(&crate::ID, "target/deploy/native_escrow_2024");

    // Set our seed
    let seed: u64 = 1337;

    // Programs
    mollusk.add_program(&spl_token::ID, "src/tests/spl_token-3.5.0");
    let (token_program, token_program_account) = (spl_token::ID, program_account(&spl_token::ID));
    let (system_program, system_program_account) = program::system_program();

    // Accounts
    let maker = Pubkey::new_from_array([0x01; 32]);
    let mint_a = Pubkey::new_from_array([0x02; 32]);
    let mint_b = Pubkey::new_from_array([0x03; 32]);
    let maker_ta_a = spl_associated_token_account::get_associated_token_address_with_program_id(
        &maker,
        &mint_a,
        &token_program,
    );
    let escrow = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &crate::ID,
    )
    .0;
    let vault = Pubkey::find_program_address(&[b"vault", escrow.as_ref()], &crate::ID).0;

    // Fill out our account data
    let mut mint_a_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x05; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_a_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut mint_b_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x06; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_b_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut maker_ta_a_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_a,
            owner: maker,
            amount: 1_000_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        maker_ta_a_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut vault_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_a,
            owner: escrow,
            amount: 0,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        vault_account.data_as_mut_slice(),
    )
    .unwrap();

    let escrow_account = AccountSharedData::new(0, 0, &Pubkey::default());

    // Create our instruction
    let instruction = Instruction::new_with_bytes(
        crate::ID,
        &[
            &[0x00],
            &seed.to_le_bytes()[..],
            &100000u64.to_le_bytes()[..],
            &100000u64.to_le_bytes()[..],
        ]
        .concat(),
        vec![
            AccountMeta::new(maker, true),
            AccountMeta::new_readonly(mint_a, false),
            AccountMeta::new_readonly(mint_b, false),
            AccountMeta::new(maker_ta_a, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    );

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &vec![
            (
                maker,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (mint_a, mint_a_account),
            (mint_b, mint_b_account),
            (maker_ta_a, maker_ta_a_account),
            (escrow, escrow_account),
            (vault, vault_account),
            (token_program, token_program_account),
            (system_program, system_program_account),
        ],
    );
    assert!(matches!(result.program_result, ProgramResult::Success))
}

#[test]
fn refund() {
    // Add our built program binary
    let mut mollusk: Mollusk = Mollusk::new(&crate::ID, "target/deploy/native_escrow_2024");

    // Set our seed
    let seed: u64 = 1337;

    // Programs
    mollusk.add_program(&spl_token::ID, "src/tests/spl_token-3.5.0");
    let (token_program, token_program_account) = (spl_token::ID, program_account(&spl_token::ID));
    let (system_program, system_program_account) = program::system_program();

    // Accounts
    let maker = Pubkey::new_from_array([0x01; 32]);
    let mint_a = Pubkey::new_from_array([0x02; 32]);
    let mint_b = Pubkey::new_from_array([0x03; 32]);
    let maker_ta_a = spl_associated_token_account::get_associated_token_address_with_program_id(
        &maker,
        &mint_a,
        &token_program,
    );
    let escrow = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &crate::ID,
    )
    .0;
    let vault = Pubkey::find_program_address(&[b"vault", escrow.as_ref()], &crate::ID).0;

    // Fill out our account data
    let mut mint_a_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x05; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_a_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut mint_b_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x06; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_b_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut maker_ta_a_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_a,
            owner: maker,
            amount: 1_000_000_000 - 100_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        maker_ta_a_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut vault_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_a,
            owner: escrow,
            amount: 100_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        vault_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut escrow_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(mem::size_of::<Escrow>()),
        mem::size_of::<Escrow>(),
        &crate::ID,
    );
    escrow_account.set_data_from_slice(bytes_of::<Escrow>(&Escrow {
        seed,
        maker,
        mint_a,
        mint_b,
        receive: 100_000,
    }));

    // Create our instruction
    let instruction = Instruction::new_with_bytes(
        crate::ID,
        &[0x02],
        vec![
            AccountMeta::new(maker, true),
            AccountMeta::new_readonly(mint_a, false),
            AccountMeta::new(maker_ta_a, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    );

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &vec![
            (
                maker,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (mint_a, mint_a_account),
            (maker_ta_a, maker_ta_a_account),
            (escrow, escrow_account),
            (vault, vault_account),
            (token_program, token_program_account),
            (system_program, system_program_account),
        ],
    );
    assert!(matches!(result.program_result, ProgramResult::Success));
}

#[test]
fn take() {
    // Add our built program binary
    let mut mollusk: Mollusk = Mollusk::new(&crate::ID, "target/deploy/native_escrow_2024");

    // Set our seed
    let seed: u64 = 1337;

    // Programs
    mollusk.add_program(&spl_token::ID, "src/tests/spl_token-3.5.0");
    let (token_program, token_program_account) = (spl_token::ID, program_account(&spl_token::ID));
    let (system_program, system_program_account) = program::system_program();

    // Accounts
    let taker = Pubkey::new_from_array([0x04; 32]);
    let maker = Pubkey::new_from_array([0x01; 32]);
    let mint_a = Pubkey::new_from_array([0x02; 32]);
    let mint_b = Pubkey::new_from_array([0x03; 32]);
    let taker_ta_a = spl_associated_token_account::get_associated_token_address_with_program_id(
        &taker,
        &mint_a,
        &token_program,
    );
    let taker_ta_b = spl_associated_token_account::get_associated_token_address_with_program_id(
        &taker,
        &mint_b,
        &token_program,
    );
    let maker_ta_b = spl_associated_token_account::get_associated_token_address_with_program_id(
        &maker,
        &mint_b,
        &token_program,
    );
    let escrow = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &crate::ID,
    )
    .0;
    let vault = Pubkey::find_program_address(&[b"vault", escrow.as_ref()], &crate::ID).0;

    // Fill out our account data
    let mut mint_a_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x05; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_a_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut mint_b_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Mint::LEN),
        spl_token::state::Mint::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_from_array([0x06; 32])),
            supply: 100_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_b_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut taker_ta_a_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_a,
            owner: taker,
            amount: 0,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        taker_ta_a_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut taker_ta_b_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_b,
            owner: taker,
            amount: 1_000_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        taker_ta_b_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut maker_ta_b_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_b,
            owner: taker,
            amount: 0,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        maker_ta_b_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut vault_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(spl_token::state::Account::LEN),
        spl_token::state::Account::LEN,
        &token_program,
    );
    solana_program::program_pack::Pack::pack(
        spl_token::state::Account {
            mint: mint_a,
            owner: escrow,
            amount: 100_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        vault_account.data_as_mut_slice(),
    )
    .unwrap();

    let mut escrow_account = AccountSharedData::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(mem::size_of::<Escrow>()),
        mem::size_of::<Escrow>(),
        &crate::ID,
    );
    escrow_account.set_data_from_slice(bytes_of::<Escrow>(&Escrow {
        seed,
        maker,
        mint_a,
        mint_b,
        receive: 100_000,
    }));

    // Create our instruction
    let instruction = Instruction::new_with_bytes(
        crate::ID,
        &[0x01],
        vec![
            AccountMeta::new(taker, true),
            AccountMeta::new(maker, false),
            AccountMeta::new_readonly(mint_a, false),
            AccountMeta::new_readonly(mint_b, false),
            AccountMeta::new(taker_ta_a, false),
            AccountMeta::new(taker_ta_b, false),
            AccountMeta::new(maker_ta_b, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(system_program, false),
        ],
    );

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &vec![
            (
                taker,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (
                maker,
                AccountSharedData::new(1_000_000_000, 0, &Pubkey::default()),
            ),
            (mint_a, mint_a_account),
            (mint_b, mint_b_account),
            (taker_ta_a, taker_ta_a_account),
            (taker_ta_b, taker_ta_b_account),
            (maker_ta_b, maker_ta_b_account),
            (escrow, escrow_account),
            (vault, vault_account),
            (token_program, token_program_account),
            (system_program, system_program_account),
        ],
    );

    assert!(matches!(result.program_result, ProgramResult::Success))
}
