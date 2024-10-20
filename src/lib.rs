pub mod block;
pub mod block_header;
pub mod block_chain;
pub mod proof_of_work;
pub mod bc_iter;
pub mod Interface;
pub mod transactions;
pub mod wallet;
pub mod functions;

pub const TARGET_BITS: u32 = 12; 
pub const MAX_NONCE: u32 = 1_000_000_000; 
pub const SUBSIDY: i32 = 70;
pub const DB_FILE: &str = "blockchain.db";








