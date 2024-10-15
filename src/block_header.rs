#[derive(Debug)]
pub struct BlockHeader {  
    // Version of the block (4 bytes)  
    version: u32,  
    // Hash of the previous block (32 bytes)  
    prev_block_hash: Vec<u8>,  
    // Merkle root (32 bytes)  
    merkle_root: Vec<u8>,  
    // Timestamp of the block (8 bytes)  
    timestamp: u64,  
    // Difficulty target (4 bytes)  
    bits: u32,  
    // Nonce (8 bytes)  
    nonce: u64,  
}