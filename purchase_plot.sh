#!/bin/bash

# Variables
keyname="${keyname:-ta0}" # Update as needed
password="${password:-12345678\n}" # Update as needed
contract="sei14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sh9m79m" # Update with your contract address
seid=~/go/bin/seid
x_coordinate=0 # Update x coordinate
y_coordinate=0 # Update y coordinate
amount="1000000usei" # Update to match the plot price

echo "Purchasing a plot at coordinates ($x_coordinate, $y_coordinate) for $amount SEI."

# Correct JSON Payload for buying a plot
buy_payload="{\"buy\":{\"coordinates\":[$x_coordinate, $y_coordinate]}}"

echo "Sending buy payload: $buy_payload"

# Purchasing a plot
purchase_resp=$(printf "$password" | $seid tx wasm execute $contract "$buy_payload" --amount $amount --from $keyname --broadcast-mode=block --chain-id sei-chain --gas=30000000 --fees=3000000usei -y)
echo "Purchase response: $purchase_resp"

# JSON Payload for querying a plot
# query_payload="{\"query_plot\":{\"coordinates\":[$x_coordinate, $y_coordinate]}}"

# echo "Querying for plot details with payload: $query_payload"

# # Fetching updated plot details
# plot_details=$(printf "$password" | $seid query wasm contract-state smart $contract "$query_payload" --output json)
# echo "Updated Plot Details: $plot_details"
