# Test script for Juno Smart Contracts (By @Reecepbcups)
# ./github/workflows/e2e.yml
#
# sh ./e2e/test_e2e.sh
#
# NOTES: anytime you use jq, use `jq -rc` for ASSERT_* functions (-c removes format, -r is raw to remove \" quotes)

# get functions from helpers file 
# -> query_contract, wasm_cmd, mint_cw721, send_nft_to_listing, send_cw20_to_listing
source ./e2e/helpers.sh

CONTAINER_NAME="tokenfactory_middleware_core"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
TX_FLAGS="--gas-prices 0.1$DENOM --gas-prices="0ujunox" --gas 5000000 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
export JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"


function create_denom {
    RANDOM_STRING=$(cat /dev/urandom | tr -dc 'a-zA-Z' | fold -w 6 | head -n 1)

    $BINARY tx tokenfactory create-denom $RANDOM_STRING $JUNOD_COMMAND_ARGS    
    export FULL_DENOM="factory/$KEY_ADDR/$RANDOM_STRING" && echo $FULL_DENOM
}

# ========================
# === Contract Uploads ===
# ========================
function upload_testing_contract {   
    echo "Storing contract..."
    UPLOAD=$($BINARY tx wasm store /tf_example.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # == INSTANTIATE ==
    # PAYLOAD=$(printf '{"core_factory_address":"%s"}' $TF_CONTRACT) && echo $PAYLOAD
    PAYLOAD=$(printf '{}' $TF_CONTRACT) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "tf_test" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH


    export TEST_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "TEST_CONTRACT: $TEST_CONTRACT"
}

function transfer_denom_to_contract {
    # transfer admin to the contract from the user
    $BINARY tx tokenfactory change-admin $FULL_DENOM $TF_CONTRACT $JUNOD_COMMAND_ARGS
    $BINARY q tokenfactory denom-authority-metadata $FULL_DENOM # admin is the TF_CONTRACT
}

function upload_tokenfactory_core {
    echo "Storing contract..."
    create_denom
    UPLOAD=$($BINARY tx wasm store /tokenfactory_core.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # == INSTANTIATE ==
    
    PAYLOAD=$(printf '{"allowed_mint_addresses":["%s"],"denoms":["%s"]}' $TEST_CONTRACT $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "tf-middlware" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH


    export TF_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "TF_CONTRACT: $TF_CONTRACT"
    
    transfer_denom_to_contract
}

# === COPY ALL ABOVE TO SET ENVIROMENT UP LOCALLY ====



# =============
# === LOGIC ===
# =============

start_docker
add_accounts
compile_and_copy # the compile takes time for the docker container to start up

sleep 5
health_status


# upload test contract
upload_testing_contract
upload_tokenfactory_core # TF_CONTRACT=juno1


# == INITIAL TEST ==
query_contract $TF_CONTRACT '{"get_config":{}}' | jq -r '.data'
# add denom
create_denom && transfer_denom_to_contract
PAYLOAD=$(printf '{"add_denom":{"denoms":["%s"]}}' $FULL_DENOM) && echo $PAYLOAD
wasm_cmd $TF_CONTRACT "$PAYLOAD" "" show_log
query_contract $TF_CONTRACT '{"get_config":{}}' | jq -r '.data.denoms'

# MINTS TOKENS FROM THE CORE CONTRACT (TF_CONTRACT) VIA THE TEST CONTRACT (TEST_CONTRACT)
PAYLOAD=$(printf '{"mint_tokens":{"core_factory_address":"%s","to_address":"%s","denoms":[{"denom":"%s","amount":"1"}]}}' $TF_CONTRACT $KEY_ADDR $FULL_DENOM) && echo $PAYLOAD
wasm_cmd $TEST_CONTRACT "$PAYLOAD" "" show_log
$BINARY q bank balances $KEY_ADDR --output json

# TODO: only allow denoms which the contract is the admin of? (query denom-authority-metadata)


# UPDATE WHITELIST ON MAIN CORE (if not in, adds. If already in, removes via the same message)
# in production, this would be another contract / a DAO
PAYLOAD=$(printf '{"add_whitelist":{"addresses":["%s"]}}' $KEY_ADDR) && echo $PAYLOAD
wasm_cmd $TF_CONTRACT "$PAYLOAD" "" show_log
query_contract $TF_CONTRACT '{"get_config":{}}' | jq -r '.data.allowed_mint_addresses'
# ensure this address can now mint tokens through the contract

PAYLOAD=$(printf '{"remove_whitelist":{"addresses":["%s"]}}' $KEY_ADDR) && echo $PAYLOAD
wasm_cmd $TF_CONTRACT "$PAYLOAD" "" show_log
query_contract $TF_CONTRACT '{"get_config":{}}' | jq -r '.data.allowed_mint_addresses'



# == TRANSFER ADMIN OF DENOM ==
# when done, we remove the denom from the denoms state as well.
PAYLOAD=$(printf '{"transfer_admin":{"denom":"%s","new_address":"juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y"}}' $FULL_DENOM) && echo $PAYLOAD
wasm_cmd $TF_CONTRACT "$PAYLOAD" "" show_log
addrs=$(query_contract $TF_CONTRACT '{"get_config":{}}' | jq -r '.data.denoms') && echo $addrs