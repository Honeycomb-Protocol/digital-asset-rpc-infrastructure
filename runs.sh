#!/usr/bin/env bash
#
# Run a minimal Solana cluster.  Ctrl-C to exit.
#
# Before running this script ensure standard Solana programs are available
# in the PATH, or that `cargo build` ran successfully
# change
#

set -e
cat << EOL > config.yaml
json_rpc_url: http://localhost:8899
websocket_url: ws://localhost:8899
commitment: finalized
EOL

mkdir plugin-config && true
if [[ ! -f /plugin-config/accountsdb-plugin-config.json ]]
then
cat << EOL > /plugin-config/accountsdb-plugin-config.json
    {
        "libpath": "/plugin/plugin.so",
        "accounts_selector" : {
            "owners" : [
                "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
                "GRoLLMza82AiYN7W9S9KCCtCyyPRAQP2ifBy4v4D5RMD",
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
                "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY",
                "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ",
                "CndyV3LdqHUfDLmE5naZjVN8rBZz4tqhdefbAnjHG3JR",
                "Guard1JwRhJkVH6XZhzoYxeBVQe872VH6QggF4BWmS9g"
            ]
        },
        "transaction_selector" : {
            "mentions" : [
                "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
                "GRoLLMza82AiYN7W9S9KCCtCyyPRAQP2ifBy4v4D5RMD",
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
                "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY",
                "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ",
                "CndyV3LdqHUfDLmE5naZjVN8rBZz4tqhdefbAnjHG3JR",
                "Guard1JwRhJkVH6XZhzoYxeBVQe872VH6QggF4BWmS9g"
            ]
        }
    }
EOL
fi

programs=()
if [ "$(ls -A /so)" ]; then
  for prog in /so/*; do
      programs+=("--bpf-program" "$(basename $prog .so)" "$prog")
  done
fi

export RUST_BACKTRACE=1
dataDir=$PWD/config/"$(basename "$0" .sh)"
ledgerDir=$PWD/config/ledger
mkdir -p "$dataDir" "$ledgerDir"
# echo $ledgerDir
# echo $dataDir
# echo ${clones[@]}
# ls -la /so/
args=(
  --config config.yaml
#   --gossip-host 145.40.103.167
  --ledger $ledgerDir
  --limit-ledger-size 1200000
  --rpc-port 8899
  --geyser-plugin-config /plugin-config/accountsdb-plugin-config.json
  --clone EtXbhgWbWEWamyoNbSRyN5qFXjFbw8utJDHvBkQKXLSL --clone UgfWEZpFdY1hPhEnuYHe24u9xR74xTSvT1uZxxiwymM # Test Hive Control
  --clone HivezrprVqHR6APKKQkkLHmUG8waZorXexEBRZWh5LRm --clone 5ZJG4CchgDXQ9LVS5a7pmib1VS69t8SSsV5riexibwTk # Hive Control
  --clone ChRCtrG7X5kb9YncA4wuyD68DXXL8Szt3zBCCGiioBTg --clone U7w6LJRtG4jvUQv4WjTinkHnv9UAfHjBiVdr2HERiX2 # Character Manager
  --clone CrncyaGmZfWvpxRcpHEkSrqeeyQsdn4MAedo9KuARAc4 --clone DcYW5MQscHQE4PmFpbohn9JJqqN3vyYau83eXTx8yAcJ # Currency Manager
  --clone Assetw8uxLogzVXic5P8wGYpVdesS1oZHfSnBFHAu42s --clone FRAhFGRAa93JwJQuHJ7HngtA1Nfp6JmSRqpGHjxtLsGK # Resource Manager
  --clone MiNESdRXUSmWY7NkAKdW9nMkjJZCaucguY3MDvkSmr6 --clone GerKtMVEu66ZCha6oaab8iGrBHc5Q6VYNRCNMgXn1WGm # Nectar Staking
  --clone Ha71K2v3q9wL3hDLbGx5mRnAXd6CdgzNx5GJHDbWvPRg --clone G5s6HRnHwRTGcE1cXAZeeCsFeurVGuW2Wqhr7UBiDZWQ
  --clone 4AZpzJtYZCu9yWrnK1D5W23VXHLgN1GPkL8h8CfaGBTW --clone 86h623JGQvvJAsPG7meWsUjFW6hBe5tLwqNPoa9baUfC
  --clone BNdAHQMniLicundk1jo4qKWyNr9C8bK7oUrzgSwoSGmZ --clone FQErtH1zXPuHRxEwamXpWG711CVhqQS3Epsv4jao4Kn1
  --clone 8fTwUdyGfDAcmdu8X4uWb2vBHzseKGXnxZUpZ2D94iit --clone FHzBQUNk6AyaSbqgS33EXcat8sXeLpvf1PJM6tQ87SPp
  --clone 9NGfVYcDmak9tayJMkxRNr8j5Ji6faThXGHNxSSRn1TK --clone 4UDQZKTAh9fo5TkC7Nh2t9tcyC7dFwFMUnrrHZLxZ1c8 
  --clone 8bvPnYE5Pvz2Z9dE6RAqWr1rzLknTndZ9hwvRE6kPDXP --clone 9xbqDdpmu47Dv2h1L3CyAtzXpMJio9bHDa5K1RWPB6LV # Libreplex fair launch
  --url https://devnet.helius-rpc.com/?api-key=141e32c2-0029-4830-ab7d-6abd61292cb3
)

# args+=("--url devnet")

# shellcheck disable=SC2086
# cat /plugin-config/accountsdb-plugin-config.json
# ls -la /so/



apt update && apt install ca-certificates -y && update-ca-certificates
solana-test-validator  "${programs[@]}" "${args[@]}" $SOLANA_RUN_SH_VALIDATOR_ARGS > /dev/null