#!/bin/bash

# Default values for variables
contract="${contract:-artifacts/counter.wasm}"
keyname="${keyname:-ta0}"
password="${password:-12345678\n}"
treasury_address="sei1nzcsve40yr5vxmrhycc4w5ra02e866t5phhfyc"
seid=~/go/bin/seid

# Storing the contract
echo "Starting to store the contract..."
store_output=$(printf $password | $seid tx wasm store $contract --from $keyname --chain-id=sei-chain --gas=10000000 --fees=10000000usei --broadcast-mode=block -o json -y)
echo "Store Contract Output: $store_output"

# Extracting code ID
code=$(echo "$store_output" | jq -r '.logs[0].events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value')

echo "Code id is: $code"

# Exiting if code ID not found
if [ -z "$code" ]; then
    echo "Failed to get code ID. Exiting."
    exit 1
fi
# Retrieving deployer address
echo "Retrieving deployer address..."
deployer_addr=$(printf $password | $seid keys show $keyname -a)
echo "Deployer address is: $deployer_addr"

# Check balance of the deployer address
echo "Checking balance of the deployer address..."
balance_output=$(printf $password | $seid query bank balances $deployer_addr)
echo "Balance of Deployer Address ($deployer_addr): $balance_output"


# Preparing instantiation message using jq
echo "Preparing instantiation message..."
init_msg=$(jq -n --arg treasury_address "$treasury_address" '{initial_price: "1000000", treasury_address: $treasury_address}')

echo "Instantiation message: $init_msg"

# Instantiating the contract with sufficient fees
echo "Instantiating the contract..."
instantiate_output=$(printf $password | $seid tx wasm instantiate "$code" "$init_msg" --label "counter" --from $keyname --admin=$deployer_addr --chain-id=sei-chain --gas=35000000 --fees=4000000usei --broadcast-mode=block -y)
echo "Instantiate Contract Output: $instantiate_output"

# Extracting deployed contract address
addr=$(echo $instantiate_output | grep -A 1 -m 1 "key: _contract_address" | sed -n 's/.*value: //p' | xargs)
echo "Deployed contract address is: $addr"

