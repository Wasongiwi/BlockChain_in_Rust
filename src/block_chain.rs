use crate::{block::Block, DB_FILE};
use crate::bc_iter::BlockchainIterator;
use std::collections::HashMap;
use crate::transactions::{Transaction, TXOutput};
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
            let cbtx = Transaction::new_coinbase_transcation(&address, &"Genesis Block".to_string());  
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
        let new_block = Block::new(transactions, last_hash.clone());
       
        self.db.insert(&new_block.hash.clone(), new_block.serialize()).expect("Failed to insert new block");   
        self.db.insert("tip", new_block.hash.clone()).expect("Failed to update tip");  
        self.tip = new_block.hash;
    }  

    pub fn iterator(&self) -> BlockchainIterator {
        BlockchainIterator::new(&self.db, self.tip.clone())  
    }

    pub fn find_utxo(&self, address: &str) -> Vec<TXOutput> {  
        let address = hex::decode(address).expect("can't decode string -> vec");
        let mut utxos = Vec::new();

        let unspent_transactions = self.find_unspent_transactions(&address);  

        for tx in unspent_transactions {  
            for out in &tx.outputs {  
                if out.is_locked_with_key(&address) {  
                    utxos.push(out.clone());  
                }  
            }  
        }  

        utxos  
    }  

    pub fn find_unspent_transactions(&self, Hash_pubKey: &Vec<u8>) -> Vec<Transaction> { 
        // print!("Finding unspent transactions for address: {}\n", address); 
        let mut unspent_txs = Vec::new();
        // 用于跟踪已花费交易输出的映射  
        let mut spent_txos: HashMap<String, Vec<usize>> = HashMap::new();
        let mut bc_iter = self.iterator();
        // print!("Current tip: {:?}\n", self.tip);
        // print!("Current hash: {:?}\n", bc_iter.current_hash.clone());
        // 遍历区块链中的每个区块  
        while let Some(block) = bc_iter.next() {  
            // print!("Processing block: {}\n", hex::encode(&block.hash));
            for transaction in &block.transactions {  
                 // 将交易 ID 编码为十六进制字符串  
                let transaction_id = hex::encode(&transaction.id);
                // print!("Processing transaction: {}\n", transaction_id);
                // 遍历交易的每个输出  
                for (out_idx, out) in transaction.outputs.iter().enumerate() {  
                    /*
                        .contains 如果找到了匹配的索引，则函数返回 true，表示当前处理的输出索引已被标记为已花费，代码将执行 continue; 跳过当前循环。
                        如果没有找到匹配，返回 false，代码将继续执行后续的逻辑，检查该输出是否能够被解锁。 

                        spent_txos 是一个 HashMap，其中键是交易 ID（十六进制编码），值是一个 Vec<usize>，表示该交易中已花费的输出索引。
                    */
                    if let Some(spent_outputs) = spent_txos.get(&transaction_id) {  
                        // 如果当前处理的输出索引已被标记为已花费，则跳过该输出  
                        if spent_outputs.contains(&out_idx) {  
                            continue;  
                        }  
                    }  
                    // 如果输出可以被解锁，则可以被花费
                    if out.is_locked_with_key(&Hash_pubKey) {  
                        //添加到未花费交易列表  
                        unspent_txs.push(transaction.clone()); 
                    }  
                }  
                // 如果该交易不是创世交易  
                if !transaction.is_coinbase() {  
                    // 遍历交易的每个输入  
                    for transaction_input in &transaction.inputs {  
                        // 如果输入可以解锁输出  
                        if transaction_input.uses_key(Hash_pubKey) {  
                            let transaction_input_id = hex::encode(&transaction_input.transcation_id); // 编码输入的交易 ID  
                            // 将输入的输出索引添加到已花费交易输出的映射中  
                            spent_txos.entry(transaction_input_id).or_insert_with(Vec::new).push(transaction_input.vout);  
                        }  
                    }  
                }  
            }  
            // 如果没有前一个区块哈希，表示区块链结束  
            if block.previous_block_hash.is_empty() {  
                break; // 结束区块链遍历  
            }  
        }  
        unspent_txs
    }     

    pub fn find_spendable_outputs(&self, hash_public_key: &Vec<u8>, amount: i32) -> (i32, HashMap<String, Vec<usize>>) {  
        // 存储可花费的输出  
        let mut unspent_outputs: HashMap<String, Vec<usize>> = HashMap::new();
        // 查找存在未花费的交易 
        let unspend_transactions = self.find_unspent_transactions(hash_public_key);  
        // 累积的金额  
        let mut accumulated = 0;

        // 遍历未花费的交易  
        'work: for transcation in unspend_transactions {  
            // 将交易 ID 编码为十六进制字符串
            let transcation_id = hex::encode(&transcation.id);  
            // 遍历交易的每个输出  
            for (out_idx, out) in transcation.outputs.iter().enumerate() {  
                // 如果输出可以被解锁且累积金额小于所需金额  
                if out.is_locked_with_key(hash_public_key) && accumulated < amount {  
                    accumulated += out.value;
                    // 添加到可花费输出映射中
                    // 将输出索引添加到未花费输出的映射中  
                    unspent_outputs.entry(transcation_id.clone()).or_insert_with(Vec::new).push(out_idx); 
                    if accumulated >= amount {  
                        break 'work; 
                    }  
                }  
            }  
        }  
        // 返回累积金额和可花费输出的映射 
        (accumulated, unspent_outputs) 
    }  

}