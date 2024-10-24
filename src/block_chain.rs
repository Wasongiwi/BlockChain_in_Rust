use crate::functions;
use crate::{block::Block, DB_FILE};
use crate::bc_iter::BlockchainIterator;
use std::clone;
use std::collections::HashMap;
use crate::transactions::{Transaction, TXOutput};
use ring::signature::EcdsaKeyPair;
use sled::{transaction, Db, IVec};  
use serde::{Serialize, Deserialize}; 

#[derive(Debug)] 
pub struct BlockChain {
    pub tip: Vec<u8>, 
    pub db: Db,
    // pub blocks: Vec<Block>,
}

impl BlockChain {

    pub fn new_blockchain(address: &String) -> BlockChain {  
        let db = sled::open(DB_FILE).expect("Failed to open database");  
        let mut tip = vec![];  

        if let Some(_) = db.get("blocks").unwrap() {  
            // 如果 bucket 存在，则从中获取 tip  
            tip = db.get("tip").expect("Failed to get tip").unwrap().to_vec();
        } else {  
            // 如果不存在块，创建创世块  
            let cbtx = Transaction::new_coinbase_transcation(address, &"Genesis Block".to_string());  
            let genesis = Self::NewGenesisBlock(cbtx);  
            // 将创世块插入到数据库  
            db.insert(genesis.hash.clone(), genesis.serialize()).expect("Failed to insert genesis block");  
            db.insert("tip", genesis.hash.clone()).expect("Failed to insert tip");  
            tip = genesis.hash;  
        }  
        BlockChain { tip, db }  
    }  

    pub fn NewGenesisBlock(coinbase: Transaction) -> Block {  
        let transactions = vec![coinbase];  
        Block::new(transactions, vec![]) // Pass an empty hash for the genesis block  
    }



    pub fn MineBlock(&mut self, transactions: Vec<Transaction>) {  
        for tx in &transactions {  
            if !self.verify_transaction(tx) {  
                panic!("ERROR: Invalid transaction");  
            }  
        }  

        let last_hash: Vec<u8>;   
        match self.db.get("tip") {  
            Ok(Some(last_hash_bytes)) => {  
                last_hash = last_hash_bytes.to_vec();
                // println!("Last hash/tip: {:?}", last_hash);  
            }  
            Ok(None) => {  
                eprintln!("Warning: Last hash not found. Creating a new genesis block.");  
                last_hash = vec![0; 32]; 
            }  
            Err(e) => {  
                panic!("Failed to get the last hash from the database: {}", e);  
            }  
        }; 

        // 更新数据库  
        let new_block = Block::new(transactions, last_hash.clone());
       
        self.db.insert(&new_block.hash.clone(), new_block.serialize()).expect("Failed to insert new block");   
        self.db.insert("tip", new_block.hash.clone()).expect("Failed to update tip");  
        self.tip = new_block.hash;

    } 

    pub fn iterator(&self) -> BlockchainIterator {
        BlockchainIterator::new(&self.db, self.tip.clone())  
    }

    pub fn find_utxo(&self, address: &str) -> Vec<TXOutput> {  
        // print!("Finding unspent transactions for address: {}\n", &address); 
        let queryPubHash_from_address = functions::address_to_pubkeyhash(address);
        let mut utxos = Vec::new();

        let unspent_transactions = self.find_unspent_transactions(&queryPubHash_from_address);  

        for tx in unspent_transactions {  
            for out in &tx.outputs {  
                if out.is_locked_with_key(&queryPubHash_from_address) {  
                    utxos.push(out.clone());  
                }  
            }  
        }  
        utxos  
    }  

    pub fn find_unspent_transactions(&self, Hash_pubKey: &Vec<u8>) -> Vec<Transaction> { 
        let mut unspent_txs = Vec::new();
        // 用于跟踪已花费交易输出的映射  
        let mut spent_txos: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        let mut bc_iter = self.iterator();
        // print!("Current tip: {:?}\n", self.tip);
        // print!("Current hash: {:?}\n", bc_iter.current_hash.clone());
        // 遍历区块链中的每个区块  
        while let Some(block) = bc_iter.next() {  
            for transaction in &block.transactions {  
                for (out_idx, out) in transaction.outputs.iter().enumerate() {  
                    // 如果这个output已经指向了一个input，那就证明这个output已经被花费
                    if let Some(spent_outputs) = spent_txos.get(&transaction.id) {  
                        if spent_outputs.contains(&out_idx) {  
                            continue;  
                        }  
                    }  
                    if out.is_locked_with_key(&Hash_pubKey) {  
                        //添加到未花费交易列表  
                        unspent_txs.push(transaction.clone()); 
                    }  
                }  
                // 如果该交易不是创世交易  
                if !transaction.is_coinbase() {  
                    for transaction_input in &transaction.inputs {  
                        if transaction_input.uses_key(Hash_pubKey) {  
                            // println!("j==================================================");
                            spent_txos.entry(transaction_input.transcation_id.clone()).or_insert_with(Vec::new).push(transaction_input.vout);  
                        }  
                    }  
                }  
            }  
            if block.previous_block_hash.is_empty() {  
                break; 
            }  
        }  
        // println!("uspent_txs {:?} \n", &unspent_txs);
        unspent_txs
    }     

    pub fn find_spendable_outputs(&self, hash_public_key: &Vec<u8>, amount: i32) -> (i32, HashMap<Vec<u8>, Vec<usize>>) {  
        // 存储可花费的输出  
        let mut unspent_outputs: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
        let unspend_transactions = self.find_unspent_transactions(hash_public_key);  
        let mut accumulated = 0;

        // 遍历这个地址未花费的交易  
        'work: for transcation in unspend_transactions {   
            // println!("==========================cur tx{:?}", &transcation);
            // 遍历交易的每个输出  
            for (out_idx, out) in transcation.outputs.iter().enumerate() {  
                // println!("==========================cur out=========={:?}", &out);
                if out.is_locked_with_key(hash_public_key) && accumulated < amount {  
                    // println!("-----------------------------{}", &out.value);
                    accumulated += out.value;
                    unspent_outputs.entry(transcation.id.clone()).or_insert_with(Vec::new).push(out_idx); 
                    if accumulated >= amount {  
                        break 'work; 
                    }  
                }  
            }  
        }  
        // 返回累积金额和可花费输出的映射 
        (accumulated, unspent_outputs) 
    }  

    pub fn find_transaction(&self, id: &Vec<u8>) -> Transaction {  
        let mut iterator = self.iterator();  
    
        while let Some(block) = iterator.next() {  
            for tx in &block.transactions {  
                if &tx.id == id {  
                    return tx.clone(); // Assuming Transaction implements Clone  
                }  
            }  
    
            if block.previous_block_hash.is_empty() {  
                break;  
            }  
        }  
    
        // Panic here if the transaction was not found.  
        panic!("Transaction not found");  
    }

    pub fn sign_transaction(&self, tx: &mut Transaction, keypair: &EcdsaKeyPair) {
        // println!("before tx.id{:?} \n", tx.id);

        let mut prev_txs = HashMap::new();  

        for vin in &tx.inputs {  
            let prev_tx = self.find_transaction(&vin.transcation_id);
            prev_txs.insert(prev_tx.id.clone(), prev_tx);  
        }  
        // println!("prec_txs {:?} \n", prev_txs);
        
        tx.sign(keypair, &prev_txs); // Assuming sign method exists in Transaction  

        // println!("after id{:?} \n ", tx.id);
    }  

    pub fn verify_transaction(&self, tx: &Transaction) -> bool {
        // println!("curr tx{:?}\n", tx);  
        let mut prev_txs = HashMap::new();  

        for vin in &tx.inputs {  
            let prev_tx = self.find_transaction(&vin.transcation_id);   
            prev_txs.insert(prev_tx.id.clone(), prev_tx);  
        }  

        tx.verify(&prev_txs) // Assuming verify method exists in Transaction  
    }  



}