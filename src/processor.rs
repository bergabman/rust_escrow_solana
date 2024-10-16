use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{instructions, state::EscrowArgs};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum EscrowInstruction {
    Make(EscrowArgs),
    Take(EscrowArgs),
    Refund(EscrowArgs),
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let instruction = EscrowInstruction::try_from_slice(data)?;

    match instruction {
        EscrowInstruction::Make(make_args) => instructions::make(program_id, accounts, make_args)?,
        EscrowInstruction::Take(take_args) => instructions::take(program_id, accounts, take_args)?,
        EscrowInstruction::Refund(refund_args) => instructions::refund(program_id, accounts, refund_args)?,
    }

    Ok(())
}
