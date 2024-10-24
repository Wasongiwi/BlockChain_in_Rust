use crate::{block_chain::BlockChain, SUBSIDY};
use crate::functions;
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
        let locking_hash = functions::publicKey_to_hash(&self.PubKey);  
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
        self.PubKeyHash = functions::address_to_pubkeyhash(&address)
    }  

    // 检查输出是否被指定的公钥哈希锁定  
    pub fn is_locked_with_key(&self, pub_key_hash: &Vec<u8>) -> bool {  
        self.PubKeyHash.cmp(&pub_key_hash) == Ordering::Equal  
        // self.PubKeyHash == pub_key_hash
    }  
    pub fn newTXOutput(value: i32, address: &String) -> TXOutput {
        let mut txo = TXOutput { 
            value, 
            PubKeyHash: Vec::new(),
        };  
        // println!("address {}", &address);
        txo.lock(&address);
        // println!("output hash{:?}", txo.PubKeyHash);
        txo 
    } 
}  


impl Transaction {  

    // IsCoinbase 判断是否是 coinbase 交易  
    pub fn is_coinbase(&self) -> bool {  
        self.inputs.len() == 1 && self.inputs[0].transcation_id.is_empty() && self.inputs[0].vout == usize::MAX - 1
    }

    pub fn set_hash(&self) -> Vec<u8> {  
        let encoded: Vec<u8> = bincode::serialize(self).expect("Error serializing transaction"); 
        let hash = Sha3_256::digest(&encoded);  
        hash.to_vec()
    } 

    pub fn set_id(&self) -> Vec<u8> {  
        let id = self.set_hash();  
        id
    }

    pub fn new_coinbase_transcation(to: &String, data: &String) -> Transaction {  
        let pubkey = data.as_bytes().to_vec();

        let txin = TXInput {  
            transcation_id: Vec::new(),  
            vout: usize::MAX - 1,  
            Signature: Vec::new(),  
            PubKey: pubkey,  
        };  
    
        let txout = TXOutput::newTXOutput(SUBSIDY, to);  
    
        let mut tx = Transaction {  
            id: Vec::new(),  
            inputs: vec![txin],  
            outputs: vec![txout],  
        };  
        
        tx.id = tx.set_id(); 
        // println!("coinbaSE Gen tx{:?}", &tx.id);
        tx  
    }

    pub fn new_utxo_transaction(from_addr: &String, to_addr: &String, amount: i32, bc: &BlockChain, cur_wallets: &Wallets) -> Transaction {  
        println!("A new transcation from: {}, to: {}, amount: {} \n", from_addr, to_addr, amount);  
        let mut inputs = Vec::new();  
        let mut outputs = Vec::new();  

        let wallet = cur_wallets.get_wallet(&from_addr).expect("can't find wallet from the address");  
        let pub_key_hash = functions::publicKey_to_hash(&wallet.public_key);  
        let (acc, valid_outputs) = bc.find_spendable_outputs(&pub_key_hash, amount);  
    
        // println!("Accumulated: {} \n, Valid Outputs: {:?} \n ", acc, valid_outputs);  
        if acc < amount {  
            panic!("ERROR: Not enough funds");  
        }  
    
        // 构建输入列表  
        for (txid, outs) in valid_outputs {  
            for &out in &outs {  
                let input = TXInput {  
                    transcation_id: txid.clone(),  
                    vout: out, 
                    Signature: Vec::new(),
                    PubKey: wallet.public_key.clone(),
                };  
                inputs.push(input);  
            }  
        }  
    
        // 构建输出列表  
        outputs.push(TXOutput::newTXOutput(amount, &to_addr));  
        
        if acc > amount {  
            outputs.push(TXOutput::newTXOutput(acc - amount, &from_addr)); 
        }  
    
        let mut tx = Transaction {  
            id: Vec::new(),  
            inputs: inputs,  
            outputs: outputs,  
        }; 
    
        tx.id = tx.set_id();  
        // println!(" ====================================sign pubkey {:?}", wallet.public_key);
        // println!(" ====================================from addr {:?}", from_addr);
        bc.sign_transaction(&mut tx, &wallet.key_pair);
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

    pub fn sign(&mut self, key_pair: &EcdsaKeyPair, prev_txs: &HashMap<Vec<u8>, Transaction>) {  
        if self.is_coinbase() {  
            return;  
        }  
        let rng = ring_rand::SystemRandom::new(); 
        let mut tx_copy = self.trimmed_copy();  
        for (in_id, vin) in self.inputs.iter_mut().enumerate() {  
            // 获取前一个交易  
            if let Some(prev_tx) = prev_txs.get(&vin.transcation_id) {  

                tx_copy.inputs[in_id].Signature = Vec::new();  
                tx_copy.inputs[in_id].PubKey = prev_tx.outputs[vin.vout].PubKeyHash.clone();  
                // tx_copy.id = tx_copy.set_id();  

                // println!("cur tx_copy {:?}\n", tx_copy);

                let signature = key_pair.sign(&rng, &tx_copy.set_hash()).unwrap();  

                vin.Signature = signature.as_ref().to_vec();  
                // vin.PubKey = Vec::new (); // 清理公钥  
            }  
        }  

        // println!(" \n self.sign {:?} \n", self.inputs[0].Signature);
        // println!(" self.pubkey {:?} \n", self.inputs[0].PubKey);
    } 

    pub fn verify(&self, prev_txs: &HashMap<Vec<u8>, Transaction>) -> bool {  
        if self.is_coinbase() {  
            return true;  
        }  
    
        for vin in &self.inputs {  
            if !prev_txs.contains_key(&vin.transcation_id) {  
                panic!("ERROR: Previous transaction is not correct");  
            }  
        }  
        // println!("prev_txs {:?} \n", prev_txs);
        let mut tx_copy = self.trimmed_copy();
        // println!("tx_copy {:?} \n", tx_copy);  
        // println!(" ver cur tx {:?} \n", self);
    
        for (in_id, vin) in self.inputs.iter().enumerate() {  
            // let prev_tx_key = hex::encode(&vin.transcation_id);  
            let prev_tx = match prev_txs.get(&vin.transcation_id) {  
                Some(tx) => tx,  
                None => {  
                    return false; // 如果找不到前一交易，返回 false  
                },  
            };  // 安全地获取前一交易  
    
            tx_copy.inputs[in_id].Signature.clear();  
            tx_copy.inputs[in_id].PubKey = prev_tx.outputs[vin.vout].PubKeyHash.clone();  
            // tx_copy.id = tx_copy.set_hash(); // 不需要在这里更新哈希因为公钥会被清空  
            // 清除公钥  
            // tx_copy.inputs[in_id].PubKey.clear(); // 不清除公钥，保持其有效性  
            // println!("ver copy {:?}", tx_copy);
    
            // 从公钥字节中提取公钥  
            let pubkey_bytes = &vin.PubKey;  
            // println!(" ver pubkey {:?}", pubkey_bytes);
            let sig = &vin.Signature;  

            let peer_public_key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, pubkey_bytes);
            return peer_public_key.verify(tx_copy.set_hash().as_slice(), sig.as_ref()).is_ok();
        }  
        true 
    }
}  



