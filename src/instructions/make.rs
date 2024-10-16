use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::{instruction::transfer_checked, state::Mint};

use crate::state::{Escrow, EscrowArgs};

pub fn make(program_id: &Pubkey, accounts: &[AccountInfo], args: EscrowArgs) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let maker = next_account_info(accounts_iter)?;
    let mint_a = next_account_info(accounts_iter)?;
    let mint_b = next_account_info(accounts_iter)?;
    let maker_ata_a = next_account_info(accounts_iter)?;
    let escrow = next_account_info(accounts_iter)?; //  PDA account to store escrow data
    let vault = next_account_info(accounts_iter)?; // AssociatedTokenAccount owned by the escrow pda to store tokens for
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;

    assert!(system_program::check_id(system_program.key));
    assert!(spl_token::check_id(token_program.key));
    assert!(spl_associated_token_account::check_id(
        associated_token_program.key
    ));
    assert!(maker.is_signer && maker.is_writable);
    // Check mint accounts
    assert!(mint_a.owner == token_program.key);
    assert!(mint_b.owner == token_program.key);
    assert!(maker_ata_a.owner == associated_token_program.key);
    assert!(maker_ata_a.is_writable);
    assert!(&mint_a.try_borrow_data()?.len() == &Mint::LEN);
    assert!(&mint_b.try_borrow_data()?.len() == &Mint::LEN);
    let mint_a_account = Mint::unpack(&mint_a.try_borrow_data()?)?;

    // Check and create the escrow
    assert!(escrow.is_writable && escrow.data_is_empty());
    let escrow_seeds = &[b"escrow", maker.key.as_ref(), &[args.escrow_bump]];
    let expected_escrow_pda = Pubkey::create_program_address(escrow_seeds, program_id)?;
    assert!(&expected_escrow_pda == escrow.key);

    invoke_signed(
        &system_instruction::create_account(
            &maker.key,
            &escrow.key,
            Rent::get()?.minimum_balance(Escrow::LEN),
            Escrow::LEN as u64,
            &program_id,
        ),
        &[maker.clone(), escrow.clone(), system_program.clone()],
        &[escrow_seeds],
    )?;

    let new_escrow = Escrow {
        seed: 0,
        maker: *maker.key,
        mint_a: *mint_a.key,
        mint_b: *mint_b.key,
        receive: args.receive,
        bump: args.escrow_bump as u64,
    };

    let mut escrow_pda_data = *bytemuck::try_from_bytes_mut::<Escrow>(*escrow.data.borrow_mut())
        .map_err(|_| ProgramError::AccountBorrowFailed)?;
    escrow_pda_data.clone_from(&new_escrow);

    // Check if vault exists. It shouldn't, as we create it now.
    assert!(vault.data_is_empty() && vault.lamports() == 0);

    invoke(
        &create_associated_token_account(maker.key, escrow.key, mint_a.key, token_program.key),
        &[
            maker.clone(),
            escrow.clone(),
            mint_a.clone(),
            // system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // Transfer amount to vault
    invoke(
        &transfer_checked(
            token_program.key,
            maker_ata_a.key,
            mint_a.key,
            vault.key,
            maker.key,
            &[maker.key],
            args.amount,
            mint_a_account.decimals,
        )?,
        &[
            maker.clone(),
            maker_ata_a.clone(),
            mint_a.clone(),
            vault.clone(),
        ],
    )?;

    Ok(())
}
