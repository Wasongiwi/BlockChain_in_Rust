use ring::{agreement::PublicKey, rand as ring_rand, signature::{self, EcdsaKeyPair, EcdsaSigningAlgorithm, KeyPair}};  
use std::fs::{self, File};  
use std::io::{self, Write, Read};  
use std::path::Path;  
use serde::{Serialize, Deserialize};
// use serde_gob::{from_reader, to_writer}; 
use sha3::{Sha3_256, Digest}; 
use rust_base58::{base58, ToBase58, FromBase58};
use std::{clone, collections::HashMap};  
use crate::functions;
use crate::DB_FILE;
use ring::signature::ECDSA_P256_SHA256_ASN1_SIGNING;
const VERSION: u8 = 0; // 假设版本号为 0  
const ADDRESS_CHECKSUM_LEN: usize = 4; // 假设地址校验和的长度为 4

// static ALGORITHM: &'static EcdsaSigningAlgorithm = &ECDSA_P256_SHA256_ASN1_SIGNING;
pub struct Wallet {  
    pub key_pair: EcdsaKeyPair,  
    pub public_key: Vec<u8>,  
}  
pub struct Wallets {  
    pub wallets: HashMap<String, Wallet>,  
}  

impl Wallet {  
    pub fn new() -> Self {  
        let (public_key, key_pair) = Self::new_key_pair(); // 获取公钥和密钥对  
        Wallet {  
            public_key,  
            key_pair,  
        }  
    }  

    fn new_key_pair() -> (Vec<u8>, EcdsaKeyPair) {  
        let rng = ring_rand::SystemRandom::new();  
        let pkcs8_document = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).expect("Failed to generate Document");  
        let pkcs8_bytes = pkcs8_document.as_ref().to_vec(); 
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &pkcs8_bytes).expect("Failed to generate key pair");  
        
        let public_key = key_pair.public_key(); // 引用公钥字节  
        let public_key_bytes: Vec<u8> = public_key.as_ref().to_vec(); 

        (public_key_bytes, key_pair)  
    }  
    
    pub fn get_hash(&self) -> Vec<u8> {  
        let mut hasher = Sha3_256::new();  
        hasher.update(&self.public_key); 
        hasher.finalize().to_vec()   
    }  

    

    pub fn get_address(&self) -> String {  
        let pub_key_hash = functions::publicKey_to_hash(&self.public_key);  
        // println!("pub_key_hash: {:?} \n", pub_key_hash);

        let mut versioned_payload = vec![VERSION];  
        versioned_payload.extend(pub_key_hash);  
        // println!("versioned_payload: {:?} \n", versioned_payload);
        
        let checksum = functions::checksum(&versioned_payload);  
        // println!("checksum: {:?} \n ", checksum);
        
        let mut full_payload = versioned_payload;  
        full_payload.extend(&checksum); // 添加校验和  
        // println!("full_payload: {:?}\n ", full_payload);
        let address = full_payload.to_base58();
        // println!("address: {:?}\n ", address);
        address  
    } 

    
}  

impl Wallets {  
    pub fn new() -> Self {  
        Wallets {  
            wallets: HashMap::new(),  
        }  
    }  
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {  
        self.wallets.get(address) 
    }  

    pub fn add_wallet(&mut self, id: String) -> &Wallet {  
        let wallet = Wallet::new();  
        self.wallets.insert(id.clone(), wallet);  // 使用 id 的 clone()  
    // 返回对钱包的引用  
        self.wallets.get(&id).expect("Wallet not found") // 使用 ID 直接查找钱包 
    }  

    pub fn new_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        println!("Your Address is:  {}", &address);
        
        self.wallets.insert(address.clone(), wallet);
        address
    }

        // 从文件加载钱包  
    // pub fn load_from_file(&mut self, wallet_file: &str) -> io::Result<()> {  
    //     if !Path::new(wallet_file).exists() {  
    //         return Err(io::Error::new(io::ErrorKind::NotFound, "Wallet file does not exist"));  
    //     }  

    //     let mut file_content = Vec::new();  
    //     let mut file = File::open(wallet_file)?;  
    //     file.read_to_end(&mut file_content)?;  

    //     let wallets: Wallets = from_reader(&file_content[..]).map_err(|e| {  
    //         io::Error::new(io::ErrorKind::InvalidData, e)  
    //     })?;  

    //     self.wallets = wallets.wallets;  

    //     Ok(())  
    // }  

    // // 保存钱包到文件  
    // pub fn save_to_db(&self){ 
    //     let db = sled::open(DB_FILE).expect("Failed to open database");  
    //     db.insert("wallets", self.serialize()).expect("Failed to insert tip"); 
    // }  

    // pub fn serialize(&self) -> Vec<u8> {  
    //     bincode::serialize(&self).unwrap()  
    // }  
    // pub fn deserialize_block(d: &[u8]) -> Block {  
    //     let block: Block = bincode::deserialize(d).expect("Failed to deserialize block");  
    //     block  
    // } 

}