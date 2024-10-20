use ripemd::Ripemd160;  
use sha3::{Sha3_256, Digest}; 




pub fn publicKey_to_hash(public_key: &Vec<u8>) -> Vec<u8> {  
    let mut hasher = Sha3_256::new();  
    hasher.update(public_key);  
    let public_sha256 = hasher.finalize();  
    
    let mut ripemd_hasher = Ripemd160::new();  
    ripemd_hasher.update(public_sha256);  
    let public_ripemd160 = ripemd_hasher.finalize();  

    public_ripemd160.to_vec() 
}  