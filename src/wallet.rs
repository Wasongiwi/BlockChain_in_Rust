extern crate rand;  
extern crate ring;  
extern crate sha3;  

use ring::{rand as ring_rand, signature::{KeyPair, EcdsaKeyPair, UnparsedPublicKey, ECDSA_P256_SHA256_FIXED}};  
use sha3::{Sha3_256, Digest}; // 导入 SHA-3 哈希算法  
use std::collections::HashMap;  

struct Wallet {  
    private_key: EcdsaKeyPair,  
    public_key: Vec<u8>,  
}  

struct Wallets {  
    wallets: HashMap<String, Wallet>,  
}  

impl Wallet {  
    pub fn new() -> Self {  
        let (private_key, public_key) = Self::new_key_pair();  
        Wallet {  
            private_key,  
            public_key,  
        }  
    }  

    fn new_key_pair() -> (EcdsaKeyPair, Vec<u8>) {  
        let rng = ring_rand::SystemRandom::new();  
        let private_key = EcdsaKeyPair::generate(&ECDSA_P256_SHA256_FIXED, &rng).expect("Failed to generate key pair");  
        let public_key_bytes = private_key.public_key().as_ref().to_vec();  
        (private_key, public_key_bytes)  
    }  
    
    pub fn get_hash(&self) -> Vec<u8> {  
        let mut hasher = Sha3_256::new();  
        hasher.update(&self.private_key.public_key()); // 你可以选择公钥或其它数据  
        hasher.finalize().to_vec() // 返回 SHA-3 哈希值  
    }  
}  

impl Wallets {  
    pub fn new() -> Self {  
        Wallets {  
            wallets: HashMap::new(),  
        }  
    }  

    pub fn add_wallet(&mut self, id: String) -> &Wallet {  
        let wallet = Wallet::new();  
        self.wallets.insert(id, wallet);  
        self.wallets.get(&id).unwrap()  
    }  
}