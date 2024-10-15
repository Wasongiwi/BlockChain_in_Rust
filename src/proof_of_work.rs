use num_bigint::BigUint;  
use num_traits::One;  
use num_traits::Zero;  
use hex;
use sha3::{Sha3_256, Digest};
use bincode::{serialize, deserialize};

use crate::block::Block;
use crate::{TARGET_BITS, MAX_NONCE};


pub struct ProofOfWork<'a> {  
    block: &'a Block,  
    target: BigUint,  
}  

impl<'a> ProofOfWork<'a> {  
    pub fn new(block: &'a Block) -> ProofOfWork<'a> {  
        // 创建目标值  
        let mut target = BigUint::one(); // 初始化为 1  
        target <<= (256 - TARGET_BITS); // 左移以设置目标  
        ProofOfWork { block, target }  
    }
    //数据合并
    pub fn prepare_data(&self, nonce: u32) -> Vec<u8> {  
        let mut data = Vec::new();  
        data.extend_from_slice(&self.block.previous_block_hash);  
        let serialized_transactions = self.block.serialize_transactions();
        data.extend_from_slice(&serialized_transactions);  
        data.extend_from_slice(&int_to_hex(self.block.timestamp.try_into().unwrap()));  
        data.extend_from_slice(&int_to_hex(TARGET_BITS as i64));  
        data.extend_from_slice(&int_to_hex(nonce as i64));  

        data  
    }    
    
    pub fn run(&self) -> (u32, Vec<u8>) {  
        let mut hash_int = BigUint::default();  
        let mut hash = [0u8; 32];  
        let mut nonce = 0;  
    // println!("Mining the block containing \"{}\"",&self.block.data);  

        while nonce < MAX_NONCE {  
            let data = self.prepare_data(nonce);
            let mut hasher = Sha3_256::new();  
            hasher.update(&data);  
            hash = hasher.finalize().into();
            hash_int = BigUint::from_bytes_be(&hash);  

            if hash_int < self.target {  
                let hash_hex = hex::encode(&hash); // 使用 hex crate 转换为字符串  
                println!("Pow: Find hash: {} from nonce: {}", hash_hex, nonce);   
                break;  
            } else {  
                nonce += 1;  
            }  
        }  
        println!("\n\n");  
        (nonce, hash.to_vec())
    }
    
    pub fn validate(&self) -> bool {  
        let mut hash_int = BigUint::default();
        let data = self.prepare_data(self.block.nonce);   
        let mut hasher = Sha3_256::new();
        hasher.update(&data);
        // 计算哈希
        let hash = hasher.finalize();   
        // 将哈希字节转换为 BigUint
        hash_int = BigUint::from_bytes_be(&hash); 
        
        hash_int < self.target  
    }    
}  

fn int_to_hex(value: i64) -> Vec<u8> {  
    let hex_string = format!("{:x}", value);  
    hex_string.into_bytes()  
}  

