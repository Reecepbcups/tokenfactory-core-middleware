# Test script for Juno Smart Contracts (By @Reecepbcups)
# ./github/workflows/e2e.yml
#
# sh ./e2e/test_e2e.sh
#
# NOTES: anytime you use jq, use `jq -rc` for ASSERT_* functions (-c removes format, -r is raw to remove \" quotes)

# get functions from helpers file 
# -> query_contract, wasm_cmd, mint_cw721, send_nft_to_listing, send_cw20_to_listing
source ./e2e/helpers.sh

CONTAINER_NAME="tokenfactory_bindings_test"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
JUNOD_CHAIN_ID='testing'
JUNOD_NODE='http://localhost:26657/'
# globalfee will break this in the future
TX_FLAGS="--gas-prices 0.1$DENOM --gas-prices="0ujunox" --gas 5000000 -y -b block --chain-id $JUNOD_CHAIN_ID --node $JUNOD_NODE --output json"
export JUNOD_COMMAND_ARGS="$TX_FLAGS --from test-user"
export KEY_ADDR="juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"


# ===================
# === Docker Init ===
# ===================
function stop_docker {
    docker kill $CONTAINER_NAME
    docker rm $CONTAINER_NAME
    docker volume rm --force junod_data
}

function start_docker {
    IMAGE_TAG=${2:-"13.0.0-beta"}
    BLOCK_GAS_LIMIT=${GAS_LIMIT:-10000000} # mirrors mainnet

    echo "Building $IMAGE_TAG"
    echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

    stop_docker    

    # run junod docker
    docker run --rm -d --name $CONTAINER_NAME \
        -e STAKE_TOKEN=$DENOM \
        -e GAS_LIMIT="$GAS_LIMIT" \
        -e UNSAFE_CORS=true \
        -e TIMEOUT_COMMIT="500ms" \
        -p 1317:1317 -p 26656:26656 -p 26657:26657 \
        --mount type=volume,source=junod_data,target=/root \
        ghcr.io/cosmoscontracts/juno:$IMAGE_TAG /opt/setup_and_run.sh $KEY_ADDR    
}

function compile_and_copy {    
    # compile vaults contract here
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/rust-optimizer:0.12.11

    # copy wasm to docker container
    docker cp ./artifacts/. $CONTAINER_NAME:/
}

function health_status {
    # validator addr
    VALIDATOR_ADDR=$($BINARY keys show validator --address) && echo "Validator address: $VALIDATOR_ADDR"

    BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR) && echo "Pre-store balance: $BALANCE_1"

    echo "Address to deploy contracts: $KEY_ADDR"
    echo "JUNOD_COMMAND_ARGS: $JUNOD_COMMAND_ARGS"
}

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
    UPLOAD=$($BINARY tx wasm store /tf_middle_test.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # == INSTANTIATE ==
    
    # PAYLOAD=$(printf '{"allowed_mint_addresses":["%s"],"denom_name":"%s"}' $KEY_ADDR $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "{}" --label "tf_test" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH


    export TEST_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "TEST_CONTRACT: $TEST_CONTRACT"
}

function upload_middlware {
    create_denom

    echo "Storing contract..."
    UPLOAD=$($BINARY tx wasm store /tokenfactory_core.wasm $JUNOD_COMMAND_ARGS | jq -r '.txhash') && echo $UPLOAD
    BASE_CODE_ID=$($BINARY q tx $UPLOAD --output json | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value') && echo "Code Id: $BASE_CODE_ID"

    # == INSTANTIATE ==
    
    PAYLOAD=$(printf '{"allowed_mint_addresses":["%s"],"denom_name":"%s"}' $TEST_CONTRACT $FULL_DENOM) && echo $PAYLOAD
    TX_HASH=$($BINARY tx wasm instantiate "$BASE_CODE_ID" "$PAYLOAD" --label "tf-middlware" $JUNOD_COMMAND_ARGS --admin "$KEY_ADDR" | jq -r '.txhash') && echo $TX_HASH


    export TF_CONTRACT=$($BINARY query tx $TX_HASH --output json | jq -r '.logs[0].events[0].attributes[0].value') && echo "TF_CONTRACT: $TF_CONTRACT"
    
    # transfer admin to the contract from the user
    $BINARY tx tokenfactory change-admin $FULL_DENOM $TF_CONTRACT $JUNOD_COMMAND_ARGS

    $BINARY q tokenfactory denom-authority-metadata $FULL_DENOM # admin is the TF_CONTRACT
}

# ===============
# === ASSERTS ===
# ===============
FINAL_STATUS_CODE=0

function ASSERT_EQUAL {
    # For logs, put in quotes. 
    # If $1 is from JQ, ensure its -r and don't put in quotes
    if [ "$1" != "$2" ]; then        
        echo "ERROR: $1 != $2" 1>&2
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS"
    fi
}
function ASSERT_CONTAINS {
    if [[ "$1" != *"$2"* ]]; then
        echo "ERROR: $1 does not contain $2" 1>&2        
        FINAL_STATUS_CODE=1 
    else
        echo "SUCCESS"
    fi
}

function add_accounts {
    # provision juno default user i.e. juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl
    echo "decorate bright ozone fork gallery riot bus exhaust worth way bone indoor calm squirrel merry zero scheme cotton until shop any excess stage laundry" | $BINARY keys add test-user --recover --keyring-backend test
    # juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk
    echo "wealth flavor believe regret funny network recall kiss grape useless pepper cram hint member few certain unveil rather brick bargain curious require crowd raise" | $BINARY keys add other-user --recover --keyring-backend test
    # juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y
    echo "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose" | $BINARY keys add user3 --recover --keyring-backend test

    # send some 10 junox funds to the users
    $BINARY tx bank send test-user juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk 10000000ujunox $JUNOD_COMMAND_ARGS
    $BINARY tx bank send test-user juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y 100000ujunox $JUNOD_COMMAND_ARGS

    # check funds where sent
    # other_balance=$($BINARY q bank balances juno1efd63aw40lxf3n4mhf7dzhjkr453axurv2zdzk --output json)
    # ASSERT_EQUAL "$other_balance" '{"balances":[{"denom":"ujunox","amount":"10000000"}],"pagination":{"next_key":null,"total":"0"}}'
}

# === COPY ALL ABOVE TO SET ENVIROMENT UP LOCALLY ====



# =============
# === LOGIC ===
# =============

start_docker
add_accounts
compile_and_copy # the compile takes time for the docker container to start up

sleep 5
# add query here until state check is good, then continue

# Don't allow errors after this point
# set -e

health_status


# new stuff

# create a denom
upload_testing_contract
upload_middlware # TF_CONTRACT=juno1


# == INITIAL TEST ==
# info=$(query_contract $TF_CONTRACT '{"contract_info":{}}' | jq -r '.data') && echo $info

# start
# MINT SOME TOKENS from the 2nd contract through middleware
PAYLOAD=$(printf '{"mint_tokens":{"tf_address":"%s","address":"%s","amount":"1"}}' $TF_CONTRACT $KEY_ADDR) && echo $PAYLOAD
wasm_cmd $TEST_CONTRACT "$PAYLOAD" "" show_log

# then check $KEY_ADDR bal
$BINARY q bank balances $KEY_ADDR --output json



wasm_cmd $TF_CONTRACT '{"change_token_admin":{"address":"juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y"}}' "" show_log
addrs=$(query_contract $TF_CONTRACT '{"info":{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"}}' | jq -r '.data') && echo $addrs

# claim
wasm_cmd $TF_CONTRACT '{"claim":{}}' "" show_log

# admin add funds
wasm_cmd $TF_CONTRACT '{"add_funds":{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl","amount":"50000000000000000000"}}' "" show_log


# addrs=$(query_contract $TF_CONTRACT '{"upgrades":{}}' | jq -r '.data') && echo $addrs

addrs=$(query_contract $TF_CONTRACT '{"points_per_block":{"address":"juno1hj5fveer5cjtn4wd6wstzugjfdxzl0xps73ftl"}}' | jq -r '.data') && echo $addrs

wasm_cmd $TF_CONTRACT '{"upgrade":{"name":"crops","num_of_times":150}}' "" show_log


wasm_cmd $TF_CONTRACT '{"unlock":{"name":"workers"}}' "" show_log
wasm_cmd $TF_CONTRACT '{"upgrade":{"name":"workers","num_of_times":86}}' "" show_log

wasm_cmd $TF_CONTRACT '{"unlock":{"name":"animals"}}' "" show_log
wasm_cmd $TF_CONTRACT '{"upgrade":{"name":"animals","num_of_times":10}}' "" show_log


# OLD
# submit price (so $1 is 1_000_000. Then when we query, we just / 1_000_000 = 1)
# only the addresses in 'addresses' can submit prices. 
# wasm_cmd $TF_CONTRACT '{"submit":{"data":[{"id":"JUNO","value":1000000}]}}' "" show_log
# wasm_cmd $TF_CONTRACT '{"submit":{"data":[{"id":"JUNO","value":1001000}]}}' "" show_log "$TX_FLAGS --keyring-backend test --from other-user"
# wasm_cmd $TF_CONTRACT '{"submit":{"data":[{"id":"JUNO","value":1004000}]}}' "" show_log "$TX_FLAGS --keyring-backend test --from user3"