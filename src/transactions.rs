use crate::{block_chain::BlockChain, SUBSIDY};
use serde::{Deserialize, Serialize};  
use sha3::{Sha3_256, Digest};

#[derive(Debug, Deserialize, Serialize, Clone)]  
pub struct Transaction {  
    pub id: Vec<u8>,  
    pub inputs: Vec<TXInput>,  
    pub outputs: Vec<TXOutput>,  
}  

#[derive(Debug, Deserialize, Serialize, Clone)] 
pub struct TXInput {  
    //指向引用的交易的 ID。
    pub transcation_id: Vec<u8>, 
    //指向那笔交易的输出的索引 
    pub vout: usize,  
    pub script_sig: String,  
}  

#[derive(Debug, Deserialize, Serialize, Clone)]   
pub struct TXOutput {  
    pub value: i32, 
    // pub ScriptPubKey: String, 
    pub script_pub_key: String,  
}  

impl TXInput {  
    // 这里的 unlockingData 可以理解为地址  
    pub fn can_unlock_output_with(&self, unlocking_data: &str) -> bool {  
        self.script_sig == unlocking_data  
    }  
}  

impl TXOutput {  
    pub fn can_be_unlocked_with(&self, unlocking_data: &str) -> bool {  
        self.script_pub_key == unlocking_data  
    }  
}  


impl Transaction {  

    // IsCoinbase 判断是否是 coinbase 交易  
    pub fn is_coinbase(&self) -> bool {  
        self.inputs.len() == 1 && self.inputs[0].transcation_id.is_empty() && self.inputs[0].vout == usize::MAX - 1
    } 

    pub fn set_id(&mut self) {  
        let encoded: Vec<u8> = bincode::serialize(self).expect("Error serializing transaction");  
        let hash = Sha3_256::digest(&encoded);  
        self.id = hash.to_vec();  
    }

    pub fn new_coinbase_transcation(to: &str, data: String) -> Transaction {  

        let txin = TXInput {  
            transcation_id: vec![],  
            vout: 0,  
            script_sig: data.clone(),  
        };  
        
        let txout = TXOutput {  
            value: SUBSIDY, // Assuming `subsidy` is defined elsewhere  
            script_pub_key: to.to_string(),  
        };  
        
        let mut tx = Transaction {  
            id: vec![],  
            inputs: vec![txin],  
            outputs: vec![txout],  
        };  
        
        tx.set_id(); // Assuming `set_id` is a method on Transaction  
    
        tx  
    }

    pub fn new_utxo_transaction(from: &str, to: &str, amount: i32, bc: &BlockChain) -> Transaction {  
        println!("A new transcation from: {}, to: {}, amount: {} \n", from, to, amount);  
        let mut inputs = Vec::new();  
        let mut outputs = Vec::new();  
    
        let (acc, valid_outputs) = bc.find_spendable_outputs(from, amount);  
    
        if acc < amount {  
            panic!("ERROR: Not enough funds");  
        }  
    
        // 构建输入列表  
        for (txid, outs) in valid_outputs {  
            let tx_id = hex::decode(&txid).expect("Invalid hex string");  
    
            for &out in &outs {  
                let input = TXInput {  
                    transcation_id: tx_id.clone(),  
                    vout: out,  
                    script_sig: from.to_string(), // 这里可以添加解锁脚本  
                };  
                inputs.push(input);  
            }  
        }  
    
        // 构建输出列表  
        outputs.push(TXOutput { 
            value: amount, 
            script_pub_key: to.to_string() });  
        
        if acc > amount {  
            outputs.push(TXOutput { 
                value: acc - amount, 
                script_pub_key: from.to_string() }); // 找零  
        }  
    
        let mut tx = Transaction {  
            id: Vec::new(),  
            inputs: inputs,  
            outputs: outputs,  
        }; 
    
        tx.set_id();  
    
        tx  
    }  
    

}  



