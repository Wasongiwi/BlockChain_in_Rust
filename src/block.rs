use sha3::{Sha3_256, Digest};
use sled::transaction;
use std::{result, time::{SystemTime, UNIX_EPOCH}};
use serde::{Serialize, Deserialize}; 
use crate::proof_of_work::ProofOfWork;
use crate::transactions::Transaction;
use crate::merkle_tree::MerkleTree;


#[derive(Serialize, Deserialize, Debug)]  
pub struct Block {  
    pub timestamp: u64,  
    pub previous_block_hash: Vec<u8>,  
    pub hash: Vec<u8>,  
    pub transactions: Vec<Transaction>,

    pub nonce: u32,
}  

impl Block {
    fn hash_transactions(&self) -> Vec<u8> {  
        let mut tx_serialized: Vec<Vec<u8>> = Vec::new();  

        for tx in &self.transactions {  
            tx_serialized.push(tx.serialize());  
        }  

        let mtree = MerkleTree::new(tx_serialized);

        mtree.root_node.unwrap().data
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