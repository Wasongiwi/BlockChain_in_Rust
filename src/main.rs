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
    cli.create_wallets();  
    let wallets = cli.wallets.as_mut().expect("Wallets not found"); // Get a mutable reference  
    println!("address1: \n");
    let address1 = wallets.new_wallet();  
    println!("address2: \n");
    let address2 = wallets.new_wallet();  
    println!("address3: \n");
    let address3 = wallets.new_wallet();  

    cli.create_blockchain(&address1);  
    
    cli.get_balance(&address1); 
    //address1 70 

    cli.send(&address1, &address2, 66); 

    cli.get_balance(&address1);  
    // address1 4
    cli.get_balance(&address2);  
    // address2 66

    cli.send(&address2, &address3, 50);
    // address2 16
    cli.get_balance(&address3);
    //address3 50
    cli.print_chain();

    // cli.print_chain(DB_FILE);

    // print_database_contents(DB_FILE); 
}



