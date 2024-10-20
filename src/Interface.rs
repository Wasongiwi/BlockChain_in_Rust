use crate::block_chain::BlockChain;
use crate::block::Block;
use crate::transactions::Transaction;
use crate::DB_FILE;
use sled::Db; // 引入 sled 数据库 
use crate::proof_of_work::ProofOfWork;
use std::fmt::{self, Debug};  
use std::env;
use std::process;  
 

pub struct CLI {  
    pub blockchain: Option<BlockChain>, // 使用 Option<BlockChain>  
}  


impl CLI {   
    pub fn new() -> Self {  
        CLI {  
            blockchain: None, // 将 blockchain 初始化为 None  
        }  
    } 

    pub fn create_blockchain(&mut self, address: &str){
        let mut bc = BlockChain::new_blockchain(&address.to_string());
        println!("Done : Creating blockchain for address: {} \n", address);
        self.blockchain = Some(bc);
        // print!("cur blockchain: {:?}", self.blockchain);
    }

    fn validate_args(&self) {  
        let args: Vec<String> = env::args().collect();  
        if args.len() < 2 {  
            self.print_usage();  
            process::exit(1);  
        }  
    }  

    fn print_usage(&self) {  
        println!("Usage: <command> <options>");  
    }  

    pub fn run(&mut self) {  

        self.validate_args();

        let args: Vec<String> = env::args().collect();  

        match args[1].as_str() {  
            "getbalance" => {  
                let address = args.get(2).expect("Address not provided");  
                self.get_balance(address);  
            }  
            "createblockchain" => {  
                let address = args.get(2).expect("Address not provided");  
                self.create_blockchain(address);  
            }  
            "printchain" => {  
                self.print_chain();  
            }  
            "send" => {  
                let from = args.get(2).expect("Source address not provided");  
                let to = args.get(3).expect("Destination address not provided");  
                let amount_str = args.get(4).expect("Amount not provided");  
                let amount: i32 = amount_str.parse().expect("Invalid amount");  

                self.send(from, to, amount);  
            }  
            _ => {  
                self.print_usage();  
                process::exit(1);  
            }  
        }  
    
    }
    pub fn get_balance(&self, address: &str) {  
        println!("Getting balance for address: {} \n", address);
        // print!("cur blockchain: {:?}", self.blockchain);
        let bc = self.blockchain.as_ref().expect("Blockchain not found");  
        let utxos = bc.find_utxo(address); // 找到未花费的交易输出  

        let balance: i32 = utxos.iter().map(|out| out.value).sum(); // 计算余额  

        println!("Balance of '{}': {}", address, balance);  
    }  

    pub fn print_chain(&self) {  
        if let Some(ref bc) = self.blockchain {
            println!("current tips {:?} \n ", bc.tip);
            let mut bci = bc.iterator();
            loop {  
                let block = bci.next(); // 获取下一个区块  

                match block {  
                    Some(block) => {  
                        // 打印区块的相关信息  
                        println!("Prev. hash: {:?}", block.previous_block_hash);  
                        // println!("transactions: {}", block.transactions);  
                        println!("Hash: {:?}", block.hash);  
                        
                        let pow = ProofOfWork::new(&block); // 创建工作量证明实例  
                        println!("PoW: {}", pow.validate()); // 打印 PoW 验证结果  
                        println!();  
                        
                        // 检查前一个区块哈希是否为空  
                        if block.previous_block_hash.is_empty() {  
                            break;  
                        }  
                    },  
                    None => {  
                        // 如果没有更多的区块  
                        break;  
                    }  
                }  
            }  
        }
    } 

    pub fn send(&mut self, from: &str, to: &str, amount: i32) {  

        if let Some(ref mut block_chain) = self.blockchain {  
            let tx = Transaction::new_utxo_transaction(from.to_string(), to.to_string(), amount, &block_chain);
            block_chain.MineBlock(vec![tx]); // 调用 MineBlock 方法  
            println!("Success send!");  
        } else {  
            eprintln!("Error: BlockChain is None");  
        }

    }  


}  