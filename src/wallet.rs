use ring::{agreement::PublicKey, rand as ring_rand, signature::{self, EcdsaKeyPair, EcdsaSigningAlgorithm, KeyPair}};  
use sha3::{Sha3_256, Digest}; 
use rust_base58::{base58, ToBase58, FromBase58};
use std::{clone, collections::HashMap};  
use crate::functions;
use ring::signature::ECDSA_P256_SHA256_ASN1_SIGNING;
const VERSION: u8 = 0; // 假设版本号为 0  
const ADDRESS_CHECKSUM_LEN: usize = 4; // 假设地址校验和的长度为 4

// static ALGORITHM: &'static EcdsaSigningAlgorithm = &ECDSA_P256_SHA256_ASN1_SIGNING;


pub struct Wallet {  
    // public_key: EcdsaKeyPair,  
    pub public_key: Vec<u8>,  
}  

pub struct Wallets {  
    pub wallets: HashMap<String, Wallet>,  
}  

impl Wallet {  
    pub fn new() -> Self {  
        let (public_key) = Self::new_key_pair();  
        Wallet {  
            public_key,  
        }  
    }  

    fn new_key_pair() -> Vec<u8> {  
        let rng = ring_rand::SystemRandom::new();  
        let pkcs8_document = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).expect("Failed to generate Document");  
        let pkcs8_bytes = pkcs8_document.as_ref().to_vec(); 
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &pkcs8_bytes).expect("Failed to generate key pair");  
        
        let public_key = key_pair.public_key(); // 引用公钥字节  
        let public_key_bytes: Vec<u8> = public_key.as_ref().to_vec(); 

        public_key_bytes  
    }  
    
    pub fn get_hash(&self) -> Vec<u8> {  
        let mut hasher = Sha3_256::new();  
        hasher.update(&self.public_key); 
        hasher.finalize().to_vec()   
    }  

    

    pub fn get_address(&self) -> Vec<u8> {  
        let pub_key_hash = functions::publicKey_to_hash(&self.public_key);  
        // println!("pub_key_hash: {:?} \n", pub_key_hash);

        let mut versioned_payload = vec![VERSION];  
        versioned_payload.extend(pub_key_hash);  
        // println!("versioned_payload: {:?} \n", versioned_payload);
        
        let checksum = self.checksum(&versioned_payload);  
        // println!("checksum: {:?} \n ", checksum);
        
        let mut full_payload = versioned_payload;  
        full_payload.extend(&checksum); // 添加校验和  
        // println!("full_payload: {:?}\n ", full_payload);
        let address: Vec<u8> = full_payload.to_base58().as_bytes().to_vec();
        // println!("address: {:?}\n ", address);
        
        address  
    } 
    fn checksum(&self, payload: &Vec<u8>) -> Vec<u8> {  
        let mut hasher = Sha3_256::new();  
        hasher.update(payload);  
        let first_sha = hasher.finalize();  
        
        let mut second_hasher = Sha3_256::new();  
        second_hasher.update(&first_sha);  
        let second_sha = second_hasher.finalize();  
        
        second_sha[..ADDRESS_CHECKSUM_LEN].to_vec()
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
}