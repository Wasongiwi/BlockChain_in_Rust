use sled::{Db};
use crate::block::Block;
pub struct BlockchainIterator<'a> {  
    pub current_hash: Vec<u8>,  
    pub db: &'a Db,  
}  

impl<'a> BlockchainIterator<'a> {  
    // 创建一个新的迭代器  
    pub fn new(db: &'a Db, start_hash: Vec<u8>) -> Self {  
        BlockchainIterator {  
            current_hash: start_hash,  
            db,  
        }  
    }  
    pub fn next(&mut self) -> Option<Block> {  
        // println!("Current hash: {:?}", self.current_hash);
        // for item in self.db.iter() {  
        //     match item {  
        //         Ok((key, value)) => {  
        //             // 打印键和值  
        //             println!("Key: {:?} \n, Value: {:?} \n", key, value);  
        //         },  
        //         Err(e) => {  
        //             eprintln!("Error retrieving item from database: {:?}", e);  
        //         },  
        //     }  
        // }
        // let result = self.db.get(&self.current_hash).expect("Failed to get block");
        // println!("Result: {:?}", result);
        if let Some(block_bytes) = self.db.get(&self.current_hash).expect("Failed to get block") {  
            // 反序列化区块  
            let block: Block = bincode::deserialize(&block_bytes).expect("Failed to deserialize block");  
            // println!("Value: {:?}", block);  
            self.current_hash = block.previous_block_hash.clone();  
            Some(block)  
        } else { 
            println!("No more blocks"); 
            None // 没有更多区块  
        }  
    }  
}  