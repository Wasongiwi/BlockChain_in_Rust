use ripemd::Ripemd160;  
use sha3::{Sha3_256, Digest}; 
use rust_base58::{base58, ToBase58, FromBase58};
use crate::ADDRESS_CHECKSUM_LEN;  
use crate::functions;





pub fn publicKey_to_hash(public_key: &Vec<u8>) -> Vec<u8> {  
    let mut hasher = Sha3_256::new();  
    hasher.update(public_key);  
    let public_sha256 = hasher.finalize();  
    
    let mut ripemd_hasher = Ripemd160::new();  
    ripemd_hasher.update(public_sha256);  
    let public_ripemd160 = ripemd_hasher.finalize();  

    public_ripemd160.to_vec() 
}  

pub fn checksum(payload: &Vec<u8>) -> Vec<u8> {  
    let mut hasher = Sha3_256::new();  
    hasher.update(payload);  
    let first_sha = hasher.finalize();  
    
    let mut second_hasher = Sha3_256::new();  
    second_hasher.update(&first_sha);  
    let second_sha = second_hasher.finalize();  
    
    second_sha[..ADDRESS_CHECKSUM_LEN].to_vec()
}

pub fn validate_address(address: &str) -> bool {  
    let pub_key_hash = address.from_base58().expect("Invalid Base58 string");  

    // 提取实际校验和  
    let actual_checksum = pub_key_hash[pub_key_hash.len() - ADDRESS_CHECKSUM_LEN..].to_vec();  

    // 提取版本字节  
    let version = pub_key_hash[0];  

    // 提取公钥哈希  
    let pub_key_hash = &pub_key_hash[1..pub_key_hash.len() - ADDRESS_CHECKSUM_LEN];  

    // 计算目标校验和  
    let target_checksum = checksum(&[version].iter().chain(pub_key_hash.iter()).cloned().collect::<Vec<u8>>());  

    // 比较实际校验和和目标校验和  
    actual_checksum == target_checksum  
}  

pub  fn address_to_pubkeyhash(address: &str) -> Vec<u8> {
    let full_payload = address.from_base58().expect("Invalid Base58 string"); 
    let pub_key_hash = full_payload[1..full_payload.len() - 4].to_vec();  

    pub_key_hash  

}  

