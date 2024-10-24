use sha3::{Sha3_256, Digest};
use sled::transaction;
use std::{result, time::{SystemTime, UNIX_EPOCH}};
use serde::{Serialize, Deserialize}; 
use crate::proof_of_work::ProofOfWork;
use crate::transactions::Transaction;


#[derive(Serialize, Deserialize, Debug)]  
pub struct Block {  
    // Timestamp of the block (8 bytes)  
    pub timestamp: u64,  
    // Hash of the previous block (32 bytes)  
    pub previous_block_hash: Vec<u8>,  
    // Hash of the block (32 bytes)  
    pub hash: Vec<u8>,  
    // Data (32 bytes)  
    pub transactions: Vec<Transaction>,

    pub nonce: u32,
}  

impl Block {
    // 计算区块里所有交易的哈希  
    fn hash_transactions(&self) -> Vec<u8> {  
        let mut tx_hashes: Vec<Vec<u8>> = Vec::new();  

        for tx in &self.transactions {  
            tx_hashes.push(tx.id.clone());  
        }  

        // 将所有的交易哈希连接起来  
        let concatenated_hashes: Vec<u8> = tx_hashes.concat();  

        // 计算连接的哈希  
        let mut hasher = Sha3_256::new();  
        hasher.update(concatenated_hashes);  
        let result = hasher.finalize();  

        result.to_vec()  
    }  

    pub fn new(transactions: Vec<Transaction>, prev_block_hash: Vec<u8>) -> Self {
        let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

        let mut block = Block {
            timestamp,
            previous_block_hash: prev_block_hash.clone(),
            transactions: transactions.clone(),
            hash: Vec::new(),
            nonce: 0,
        };
        let pow = ProofOfWork::new(&block);

        (block.nonce, block.hash) = pow.run();
        
        return block;
    }
    //序列化
    pub fn serialize(&self) -> Vec<u8> {  
        bincode::serialize(&self).unwrap()  
    }  
    pub fn deserialize_block(d: &[u8]) -> Block {  
        let block: Block = bincode::deserialize(d).expect("Failed to deserialize block");  
        block  
    }  
    pub fn serialize_transactions(&self) -> Vec<u8> {  
        bincode::serialize(&self.transactions).unwrap() 
    }  

}