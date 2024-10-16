use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::{
    instruction::{close_account, transfer_checked},
    state::Mint,
};

use crate::state::{Escrow, EscrowArgs};

pub fn take(program_id: &Pubkey, accounts: &[AccountInfo], args: EscrowArgs) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let taker = next_account_info(accounts_iter)?;
    let maker = next_account_info(accounts_iter)?;
    let mint_a = next_account_info(accounts_iter)?;
    let mint_b = next_account_info(accounts_iter)?;
    let taker_ata_a = next_account_info(accounts_iter)?;
    let taker_ata_b = next_account_info(accounts_iter)?;
    let maker_ata_b = next_account_info(accounts_iter)?;
    let escrow = next_account_info(accounts_iter)?;
    let vault = next_account_info(accounts_iter)?;
    // let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;

    // assert!(system_program::check_id(system_program.key));
    assert!(spl_token::check_id(token_program.key));
    assert!(spl_associated_token_account::check_id(
        associated_token_program.key
    ));

    assert!(taker.is_signer && taker.is_writable);
    assert!(maker.is_writable);
    assert!(mint_a.owner == token_program.key);
    assert!(mint_b.owner == token_program.key);
    assert!(taker_ata_a.owner == associated_token_program.key);
    assert!(taker_ata_a.is_writable);
    assert!(maker_ata_b.owner == associated_token_program.key);
    assert!(maker_ata_b.is_writable);
    assert!(&mint_a.try_borrow_data()?.len() == &Mint::LEN);
    assert!(&mint_b.try_borrow_data()?.len() == &Mint::LEN);
    let mint_a_account = Mint::unpack(&mint_a.try_borrow_data()?)?;
    assert!(escrow.owner == program_id);

    let escrow_data: Escrow = *bytemuck::try_from_bytes::<Escrow>(*escrow.data.borrow())
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    assert!(escrow_data.mint_a == *mint_a.key);
    assert!(escrow_data.mint_b == *mint_b.key);
    assert!(escrow_data.maker == *maker.key);

    let escrow_seeds = &[b"escrow", maker.key.as_ref(), &[escrow_data.bump as u8]];

    if taker_ata_a.data_is_empty() && taker_ata_a.lamports() == 0 {
        invoke(
            &create_associated_token_account(taker.key, taker.key, mint_a.key, token_program.key),
            &[
                taker.clone(),
                mint_a.clone(),
                // system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
            ],
        )?;
    }

    // Transfer amount A from vault to taker
    invoke_signed(
        &transfer_checked(
            token_program.key,
            vault.key,
            mint_a.key,
            taker_ata_a.key,
            program_id,
            &[],
            args.amount,
            mint_a_account.decimals,
        )?,
        &[
            taker.clone(),
            taker_ata_a.clone(),
            mint_a.clone(),
            vault.clone(),
        ],
        &[escrow_seeds],
    )?;

    // Transfer amount B from taker to maker
    invoke(
        &transfer_checked(
            token_program.key,
            taker_ata_b.key,
            mint_b.key,
            maker_ata_b.key,
            taker.key,
            &[taker.key],
            args.receive,
            mint_a_account.decimals,
        )?,
        &[
            taker.clone(),
            maker.clone(),
            taker_ata_b.clone(),
            maker_ata_b.clone(),
            mint_b.clone(),
        ],
    )?;

    // Close escrow
    let mut data = escrow.data.borrow_mut();
    data.fill(0);
    let maker_orig_lamports = maker.lamports();
    **maker.lamports.borrow_mut() = maker_orig_lamports.checked_add(escrow.lamports()).ok_or(ProgramError::ArithmeticOverflow)?;
    **escrow.lamports.borrow_mut() = 0;

    // Close vault
    invoke_signed(
        &close_account(
            token_program.key, 
            vault.key,
            maker.key,
            escrow.key, 
            &[])?,
        &[
            escrow.clone(), 
            vault.clone(), 
            maker.clone()],
        &[escrow_seeds],
    )?;

    Ok(())
}
