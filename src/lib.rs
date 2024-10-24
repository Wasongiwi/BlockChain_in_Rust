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
pub const GENESIS: i32 = 77;
pub const SUBSIDY: i32 = 10;
pub const DB_FILE: &str = "blockchain.db";
const VERSION: u8 = 0; // 假设版本号为 0  
const ADDRESS_CHECKSUM_LEN: usize = 4; // 假设地址校验和的长度为 4








