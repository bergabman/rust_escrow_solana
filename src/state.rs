use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Escrow {
    pub seed: u64,
    pub receive: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub bump: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EscrowArgs {
    pub maker: Pubkey,
    pub taker: Option<Pubkey>,
    pub amount: u64,
    pub receive: u64,
    pub escrow_bump: u8,
}

impl Escrow {
    // pub const LEN: usize = std::mem::size_of::<Escrow>();
    pub const LEN: usize = core::prelude::rust_2021::size_of::<u64>()
        + std::mem::size_of::<Pubkey>()
        + std::mem::size_of::<Pubkey>()
        + std::mem::size_of::<u64>()
        + std::mem::size_of::<u8>();
}
