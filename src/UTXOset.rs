use sled::Db;
use crate::block_chain::BlockChain;
use crate::DB_FILE;
use crate::block::Block;
use crate::functions;

use std::collections::HashMap;  
use std::error::Error;  
use hex;  
use crate::transactions::{TXInput, TXOutput, TXOutputs};

pub struct UTXOSet {  
    pub blockchain: BlockChain,  
}  


impl UTXOSet {  

    pub fn update(&self, block: &Block) {  
        let db = &self.blockchain.db;  
        let bucket = db.open_tree("utxoBucket").expect("Failed to open utxoBucket");  

            for transaction in &block.transactions {  
                if !transaction.is_coinbase() {  
                    for vin in &transaction.inputs {  
                        let mut updated_outs = TXOutputs { outputs: vec![] };  
                        
                        if let Some(outs_bytes) = bucket.get(&vin.transcation_id).unwrap() { 
                            let outs = match bincode::deserialize::<TXOutputs>(&outs_bytes) {  
                                Ok(outs) => outs,  
                                Err(e) => {  
                                    // log::error!("Failed to deserialize TXOutputs: {:?}", e);  
                                    continue;   
                                },  
                            };  

                            for (out_idx, out) in outs.outputs.iter().enumerate() {  
                                if out_idx != vin.vout {  
                                    updated_outs.outputs.push(out.clone());  
                                }  
                            }  
                        } else {  
                            continue;  
                        }  
                        
                        if updated_outs.outputs.is_empty() {  
                            bucket.remove(&vin.transcation_id).expect("Failed to delete UTXO");  
                        } else {  
                            bucket.insert(&vin.transcation_id, updated_outs.serialize()).expect("Failed to update UTXO");  
                        }  
                    }  
                }  
                let new_outputs = TXOutputs {  
                    outputs: transaction.outputs.clone(),  
                };  
                bucket.insert(&transaction.id, new_outputs.serialize()).expect("Failed to insert new UTXO");  
            }  
    }

    pub fn find_utxos(&self, address: &str) -> Vec<TXOutput> {  
        let blocks_bucket = self.blockchain.db.open_tree("blocks").expect("Failed to open blocks tree");  
        let queryPubHash_from_address = functions::address_to_pubkeyhash(address);
        let mut utxos = Vec::new();  

        for result in blocks_bucket.iter() {  
            if let Ok((key, value)) = result {  
                let block: Block = bincode::deserialize(&value).expect("Failed to deserialize Block");  

                for transaction in block.transactions {  
                    for output in transaction.outputs {  
                        if output.is_locked_with_key(&queryPubHash_from_address) {  
                            utxos.push(output.clone());  
                        }  
                    }  
                }  
            }  
        }  
        utxos  
    }   
    
    pub fn find_spendable_outputs(
        &self, 
        pubkey_hash: &Vec<u8>, 
        amount: i32
    ) -> (i32, HashMap<Vec<u8>, Vec<usize>>) {  
        let utxo_bucket = self.blockchain.db.open_tree("utxoBucket").expect("Failed to open UTXO bucket");  
        let mut unspent_outputs: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();  
        let mut accumulated: i32 = 0;  
        for result in utxo_bucket.iter() {  
            if let Ok((key, value)) = result {  
                let tx_id = key.to_vec();  
                let outs: TXOutputs = bincode::deserialize(&value).expect("Failed to deserialize Outputs"); 
    
                for (out_idx, out) in outs.outputs.iter().enumerate() {  
                    if out.is_locked_with_key(pubkey_hash) && accumulated < amount {  
                        accumulated += out.value;  
                        unspent_outputs.entry(tx_id.clone()).or_insert_with(Vec::new).push(out_idx);  
                    }  
                }  
                if accumulated >= amount {  
                    break;  
                }  

            }
        }(accumulated, unspent_outputs)  
    }  


    pub fn count_transactions(&self) -> usize {  
        let db = &self.blockchain.db;  
        let mut counter = 0;  

        let bucket = db.open_tree("utxoBucket").expect("Failed to open UTXO bucket");  
            for _ in bucket.iter() {  
                counter += 1; // 每个输出计数  
            }  
        counter  
    }  

    // 重新索引 UTXO 集  
    pub fn reindex(&self) {  
        // println!("1 \n");
        let db = &self.blockchain.db;  
        let bucket_name = "utxoBucket";  

        db.drop_tree(bucket_name);

        let bucket = db.open_tree("utxoBucket").expect("Failed to create UTXO bucket");  

        let utxo = self.blockchain.find_utxo(); 
        // println!("find utxo {:?}\n", utxo);
        for (tx_id, outs) in utxo {  
            let key = tx_id;
            let encoded_outputs = outs.serialize(); 
            bucket.insert(key, encoded_outputs).expect("Failed to insert output");  
        }  
    }  
}
