#!/bin/bash  
echo "Adding block: Send 1 BTC to Ivan"  
cargo run -- addblock -data "Send 1 BTC to Ivan"  
echo "Mining the block containing \"Send 1 BTC to Ivan\""  

# 添加第二个区块  
echo "Adding block: Pay 0.31337 BTC for a coffee"  
cargo run -- addblock -data "Pay 0.31337 BTC for a coffee"  
echo "Mining the block containing \"Pay 0.31337 BTC for a coffee\""  

# 打印整个区块链  
echo "Printing the blockchain"  
cargo run -- printchain