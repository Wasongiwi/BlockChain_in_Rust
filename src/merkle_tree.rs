use sha3::{Sha3_256, Digest};  

#[derive(Debug)]  
pub struct MerkleTree {  
    pub root_node: Option<MerkleNode>,  
}  

// MerkleNode represents a Merkle tree node  
#[derive(Debug, Clone)]  
pub struct MerkleNode {  
    left: Option<Box<MerkleNode>>,  
    right: Option<Box<MerkleNode>>,  
    pub data: Vec<u8>,  
}  

impl MerkleTree {  
    // Creates a new Merkle tree from a sequence of data  
    pub fn new(data: Vec<Vec<u8>>) -> MerkleTree {  
        let mut nodes: Vec<MerkleNode> = Vec::new();  

        let mut data = data;  

        if data.len() % 2 != 0 {  
            data.push(data[data.len() - 1].clone());  
        }  

        for datum in data {  
            let node = MerkleNode::new(None, None, datum);  
            nodes.push(node);  
        }  

        // Build tree from leaf nodes  
        while nodes.len() > 1 {  
            let mut new_level: Vec<MerkleNode> = Vec::new();  

            for i in (0..nodes.len()).step_by(2) {  
                let left = Some(Box::new(nodes[i].clone()));  
                let right = if i + 1 < nodes.len() {  
                    Some(Box::new(nodes[i + 1].clone()))  
                } else {  
                    None  
                };  
                let node = MerkleNode::new(left, right, Vec::new());  
                new_level.push(node);  
            }  

            nodes = new_level;  
        }  

        MerkleTree {  
            root_node: nodes.into_iter().next(),  
        }  
    }  
}  

impl MerkleNode {  
    // Creates a new Merkle tree node  
    pub fn new(left: Option<Box<MerkleNode>>, right: Option<Box<MerkleNode>>, data: Vec<u8>) -> MerkleNode {  
        let mut m_node = MerkleNode {  
            left,  
            right,  
            data: Vec::new(),  
        };  

        if m_node.left.is_none() && m_node.right.is_none() {  
            // Leaf node: hash the data  
            let mut hasher = Sha3_256::new();  
            hasher.update(&data);  
            m_node.data = hasher.finalize().to_vec();  
        } else {  
            // Non-leaf node: hash the combined hashes of children  
            if let (Some(left), Some(right)) = (&m_node.left, &m_node.right) {  
                let mut prev_hashes = Vec::new();  
                prev_hashes.extend(&left.data);  
                prev_hashes.extend(&right.data);  
                
                let mut hasher = Sha3_256::new();  
                hasher.update(&prev_hashes);  
                m_node.data = hasher.finalize().to_vec();  
            }  
        }  

        m_node  
    }  
    pub fn print(&self, level: usize) {  
        let indent = "    ".repeat(level);  
        // 打印当前节点的哈希值  
        println!("{}Node Hash: {:?}", indent, hex::encode(&self.data));  

        // 递归打印左右子节点  
        if let Some(ref left) = self.left {  
            left.print(level + 1);  
        }  
        if let Some(ref right) = self.right {  
            right.print(level + 1);  
        }  
    } 
}

