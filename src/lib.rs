use solana_program::entrypoint;

mod instructions;
mod processor;
mod state;
use processor::process_instruction;

entrypoint!(process_instruction);
