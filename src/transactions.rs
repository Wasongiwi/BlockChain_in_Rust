use crate::{block_chain::BlockChain, SUBSIDY};
use crate::functions::publicKey_to_hash ;
use crate::wallet::Wallets;

use ring::{rand as ring_rand, signature::{self, EcdsaKeyPair, UnparsedPublicKey, Signature, ECDSA_P256_SHA256_ASN1}};

use serde::{Deserialize, Serialize};  
use sha3::{Sha3_256, Digest};
use std::cmp::Ordering;  
use rust_base58::{base58, ToBase58, FromBase58};
use std::collections::HashMap; 

// use ring::signature::ECDSA_P256_SHA256_ASN1;



#[derive(Debug, Deserialize, Serialize, Clone)]  
pub struct Transaction {  
    pub id: Vec<u8>,  
    pub inputs: Vec<TXInput>,  
    pub outputs: Vec<TXOutput>,  
}  

#[derive(Debug, Deserialize, Serialize, Clone)] 
pub struct TXInput {  
    pub transcation_id: Vec<u8>, 
    pub vout: usize,  
    pub Signature: Vec<u8>,
	pub PubKey: Vec<u8>,
}  

#[derive(Debug, Deserialize, Serialize, Clone)]   
pub struct TXOutput {  
    pub value: i32, 
    // pub ScriptPubKey: String, 
    pub PubKeyHash:Vec<u8>,
}  

impl TXInput {  
    // 这里的 unlockingData 可以理解为地址  
    // pub fn can_unlock_output_with(&self, unlocking_data: &str) -> bool {  
    //     self.script_sig == unlocking_data  
    // }  

    pub fn uses_key(&self, pub_key_hash: &Vec<u8>) -> bool {  
        let locking_hash = publicKey_to_hash(&self.PubKey);  
        locking_hash.cmp(&pub_key_hash) == Ordering::Equal 

        // pub_key_hash == locking_hash 
    }  
}  

impl TXOutput {  
    // pub fn can_be_unlocked_with(&self, unlocking_data: &str) -> bool {  
    //     self.script_pub_key == unlocking_data  
    // }  

    // 将地址锁定到输出  
    pub fn lock(&mut self, address: &String) {  
        let addr_vec = hex::decode(&address).expect("can't decode to vec");
        let pub_key_hash = addr_vec.from_base58().expect("can't  decode base58"); // 假设有一个 Base58 解码函数  
        self.PubKeyHash = pub_key_hash[1..pub_key_hash.len() - 4].to_vec(); // 提取公钥哈希  
    }  

    // 检查输出是否被指定的公钥哈希锁定  
    pub fn is_locked_with_key(&self, pub_key_hash: &Vec<u8>) -> bool {  
        self.PubKeyHash.cmp(&pub_key_hash) == Ordering::Equal  
        // self.PubKeyHash == pub_key_hash
    }  
    pub fn newTXOutput(value: i32, address: &String) -> TXOutput {
        let mut txo = TXOutput { 
            value, 
            PubKeyHash: vec![],
        };  
        txo.lock(&address); // Convert address to bytes and lock it  
        txo 
    } 
}  


impl Transaction {  

    // IsCoinbase 判断是否是 coinbase 交易  
    pub fn is_coinbase(&self) -> bool {  
        self.inputs.len() == 1 && self.inputs[0].transcation_id.is_empty() && self.inputs[0].vout == usize::MAX - 1
    }

    pub fn set_Serialize(&self) -> Vec<u8> {  
        let encoded: Vec<u8> = bincode::serialize(self).expect("Error serializing transaction"); 
        encoded  
    } 

    pub fn set_hash(&self) -> Vec<u8> {  
        let encoded = self.set_Serialize();  
        let hash = Sha3_256::digest(&encoded);  
        let id = hash.to_vec();  

        id
    }

    pub fn new_coinbase_transcation(to: &String, data: &String) -> Transaction {  
        let pubkey = hex::decode(data).expect("can't decode string -> vec");

        let txin = TXInput {  
            transcation_id: Vec::new(),  
            vout: usize::MAX - 1,  
            Signature: Vec::new(),  
            PubKey: pubkey,  
        };  
    
        let txout = TXOutput::newTXOutput(SUBSIDY, &to);  
    
        let mut tx = Transaction {  
            id: Vec::new(),  
            inputs: vec![txin],  
            outputs: vec![txout],  
        };  
        
        tx.id = tx.set_hash(); 
    
        tx  
    }

    pub fn new_utxo_transaction(from_addr: String, to: String, amount: i32, bc: &BlockChain) -> Transaction {  
        println!("A new transcation from: {}, to: {}, amount: {} \n", from_addr, to, amount);  
        let mut inputs = Vec::new();  
        let mut outputs = Vec::new();  

        let wallets = Wallets::new();  
        let wallet = wallets.get_wallet(&from_addr).expect("can't find wallet");  
    
        let pub_key_hash = publicKey_to_hash(&wallet.public_key);  
    
        // 假设有一个区块链实例  
        let (acc, valid_outputs) = bc.find_spendable_outputs(&pub_key_hash, amount);  
    
        println!("Accumulated: {}, Valid Outputs: {:?}", acc, valid_outputs);  
        if acc < amount {  
            panic!("ERROR: Not enough funds");  
        }  
    
        // 构建输入列表  
        for (txid, outs) in valid_outputs {  
            let tx_id = hex::decode(&txid).expect("Invalid tx_ID");  
    
            for &out in &outs {  
                let input = TXInput {  
                    transcation_id: tx_id.clone(),  
                    vout: out, 
                    Signature: Vec::new(),
                    PubKey: wallet.public_key.clone(),
                };  
                inputs.push(input);  
            }  
        }  
    
        // 构建输出列表  
        outputs.push(TXOutput::newTXOutput(amount, &to));  
        
        if acc > amount {  
            outputs.push(TXOutput::newTXOutput(acc - amount, &to)); 
        }  
    
        let mut tx = Transaction {  
            id: Vec::new(),  
            inputs: inputs,  
            outputs: outputs,  
        }; 
    
        tx.set_Serialize();  
    
        tx  
    }  
    


    fn trimmed_copy(&self) -> Transaction {  
        let inputs: Vec<TXInput> = self.inputs.iter()  
            .map(|vin| TXInput {  
                transcation_id: vin.transcation_id.clone(),  
                vout: vin.vout,  
                Signature: Vec::new(),    // 清除签名  
                PubKey: Vec::new(),       // 清除公钥  
            })  
            .collect();  

        let outputs: Vec<TXOutput> = self.outputs.iter()  
            .map(|vout| TXOutput {  
                value: vout.value,  
                PubKeyHash: vout.PubKeyHash.clone(),  
            })  
            .collect();  

        Transaction {  
            id: self.id.clone(),  
            inputs: inputs,  
            outputs: outputs,  
        }  
    }  

    pub fn sign(&mut self, key_pair: &EcdsaKeyPair, prev_txs: &HashMap<String, Transaction>) {  
        if self.is_coinbase() {  
            return;  
        }  
        let rng = ring_rand::SystemRandom::new(); 
        let mut tx_copy = self.trimmed_copy();  
        // 遍历每个输入  
        for (in_id, vin) in self.inputs.iter_mut().enumerate() {  
            // 获取前一个交易  
            if let Some(prev_tx) = prev_txs.get(&hex::encode(&vin.transcation_id)) {  
                tx_copy.inputs[in_id].Signature = Vec::new();  
                tx_copy.inputs[in_id].PubKey = prev_tx.outputs[vin.vout].PubKeyHash.clone();  

                // 更新交易ID的哈希  
                tx_copy.set_hash(); // 更新哈希  

                // 进行签名  
                let signature = key_pair.sign(&rng, &tx_copy.set_hash()).unwrap();  

                // 将签名存储到输入中  
                vin.Signature = signature.as_ref().to_vec();  
                vin.PubKey = Vec::new(); // 清理公钥  
            }  
        }  
    } 

    pub fn verify(&self, prev_txs: &HashMap<String, Transaction>) -> bool {  
        if self.is_coinbase() {  
            return true;  
        }  
    
        for vin in &self.inputs {  
            let prev_tx_key = hex::encode(&vin.transcation_id);  
            if !prev_txs.contains_key(&prev_tx_key) {  
                panic!("ERROR: Previous transaction is not correct");  
            }  
        }  
    
        let mut tx_copy = self.trimmed_copy();  
    
        for (in_id, vin) in self.inputs.iter().enumerate() {  
            let prev_tx_key = hex::encode(&vin.transcation_id);  
            let prev_tx = match prev_txs.get(&prev_tx_key) {  
                Some(tx) => tx,  
                None => {  
                    return false; // 如果找不到前一交易，返回 false  
                },  
            };  // 安全地获取前一交易  
    
            // 清除签名  
            tx_copy.inputs[in_id].Signature.clear();  
            // 确保获取前一交易输出的公钥哈希  
            tx_copy.inputs[in_id].PubKey = prev_tx.outputs[vin.vout].PubKeyHash.clone();  
            // tx_copy.id = tx_copy.set_hash(); // 不需要在这里更新哈希因为公钥会被清空  
            // 清除公钥  
            // tx_copy.inputs[in_id].PubKey.clear(); // 不清除公钥，保持其有效性  
    
            // 提取签名并分割为 R 和 S  
            let signature = &vin.Signature;  
            let sig_len = signature.len();  
            let r = &signature[..sig_len / 2];   
            let s = &signature[sig_len / 2..];  
    
            // 确保 R 和 S 的长度符合 ECDSA 签名的长度要求  
            if r.len() != 32 || s.len() != 32 {  
                return false; // 如果 R/S 的长度不合法，返回 false  
            }  
    
            // 从公钥字节中提取公钥  
            let pubkey_bytes = &vin.PubKey;  
    
            // 创建公钥并进行签名验证  
            let raw_pubkey = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, pubkey_bytes);  
            
            // DER编码签名  
            let mut der_signature = Vec::new();  
            der_signature.push(0x30); // SEQUENCE  
            let r_len = r.len();  
            let s_len = s.len();  
            der_signature.push((r_len + s_len + 4) as u8); // Length of R + S + two tags  
            der_signature.push(0x02); // INTEGER for R  
            der_signature.push(r_len as u8);  
            der_signature.extend_from_slice(r);  
            der_signature.push(0x02); // INTEGER for S  
            der_signature.push(s_len as u8);  
            der_signature.extend_from_slice(s);  
    
            // 使用 DER 编码签名进行验证  
            if raw_pubkey.verify(&tx_copy.id, &der_signature).is_err() {  
                return false; // 如果验证失败，返回 false  
            }  
        }  
        true // 如果所有签名都有效，返回 true  
    }
}  



