use sled::Db; // 引入 sled 数据库  
use Blockchain_in_Rust::DB_FILE;
use Blockchain_in_Rust::{Interface::CLI, block_chain::BlockChain,block::Block};

fn print_database_contents(file_db: &str) {  
    let db = sled::open(file_db).expect("Failed to open the database"); 
    // 遍历数据库中的所有键值对  
    for result in db.iter() {  
        match result {  
            Ok((key, value)) => {  
                // 打印键  
                println!("Key: {:?}", key);  
                
                // 调整后的反序列化代码  
                if let Ok(block) = bincode::deserialize::<Block>(&value) {  
                    println!("Value: {:?}", block);  
                } else {  
                    println!("Failed to deserialize value");  
                }  
            }  
            Err(err) => {  
                println!("Error reading from database: {:?}", err);  
            }  
        }  
    }  
    println!("Database contents printed.");
}
fn main() {  
    
    let mut cli = CLI::new();  

    // 第一步：创建区块链  
    cli.create_blockchain("lihuacheng");  
    
    // 第二步：获取余额  
    cli.get_balance("lihuacheng"); 
    cli.send("lihuacheng", "lihua", 66);
    cli.get_balance("lihuacheng"); 
    cli.get_balance("lihua"); 
    cli.print_chain();

    // cli.print_chain(DB_FILE);

    // print_database_contents(DB_FILE); 
}



