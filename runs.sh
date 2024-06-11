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
if [[ ! -f /plugin-config/grpc-plugin-config.json ]]
then
cat << EOL > /plugin-config/grpc-plugin-config.json
    {
        "libpath": "/plugin/plugin.so",
        "log": {
            "level": "info"
        },
        "grpc": {
            "address": "0.0.0.0:10000",
            "max_decoding_message_size": "4_194_304",
            "snapshot_plugin_channel_capacity": null,
            "snapshot_client_channel_capacity": "50_000_000",
            "channel_capacity": "100_000",
            "unary_concurrency_limit": 100,
            "unary_disabled": false,
            "filters": {
                "accounts": {
                    "max": 1,
                    "any": false,
                    "owner_reject": [
                        "11111111111111111111111111111111"
                    ]
                },
                "slots": {
                    "max": 1
                },
                "transactions": {
                    "max": 1,
                    "any": false
                },
                "transactions_status": {
                    "max": 1
                },
                "blocks": {
                    "max": 1,
                    "account_include_max": 10,
                    "account_include_any": false,
                    "account_include_reject": [
                        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                    ],
                    "include_transactions": true,
                    "include_accounts": false,
                    "include_entries": false
                },
                "blocks_meta": {
                    "max": 1
                },
                "entry": {
                    "max": 1
                }
            }
        },
        "prometheus": {
            "address": "0.0.0.0:8999"
        },
        "block_fail_action": "log"
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
  --ledger $ledgerDir
  --limit-ledger-size 1200000
  --rpc-port 8899
  --geyser-plugin-config /plugin-config/grpc-plugin-config.json
  --clone EtXbhgWbWEWamyoNbSRyN5qFXjFbw8utJDHvBkQKXLSL --clone UgfWEZpFdY1hPhEnuYHe24u9xR74xTSvT1uZxxiwymM # Test Hive Control
#   --clone HivezrprVqHR6APKKQkkLHmUG8waZorXexEBRZWh5LRm --clone 5ZJG4CchgDXQ9LVS5a7pmib1VS69t8SSsV5riexibwTk # Hive Control
#   --clone ChRCtrG7X5kb9YncA4wuyD68DXXL8Szt3zBCCGiioBTg --clone U7w6LJRtG4jvUQv4WjTinkHnv9UAfHjBiVdr2HERiX2 # Character Manager
#   --clone CrncyaGmZfWvpxRcpHEkSrqeeyQsdn4MAedo9KuARAc4 --clone DcYW5MQscHQE4PmFpbohn9JJqqN3vyYau83eXTx8yAcJ # Currency Manager
#   --clone RSCR7UoY65mDMK8z2eCBvFmj4HSepGEY9ZjdCTiUDUA --clone 85rdQei5HBMjFoN1ProxSu5kC2LuyKXhaRqRRtAb9w9u # Resource Manager
#   --clone MiNESdRXUSmWY7NkAKdW9nMkjJZCaucguY3MDvkSmr6 --clone GerKtMVEu66ZCha6oaab8iGrBHc5Q6VYNRCNMgXn1WGm # Nectar Staking
#   --clone B4DxK2DhseG2ieSqckSoSvfUbYRz6hbNfXeWwmF7dm4h --clone DhZGz5bbfvqeMn9tb6FbH4Q1jpcb1K9REon5Ps92GuMV # Resource Manager TEST
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
# cat /plugin-config/grpc-plugin-config.json
# ls -la /so/



apt update && apt install ca-certificates -y && update-ca-certificates
solana-test-validator  "${programs[@]}" "${args[@]}" $SOLANA_RUN_SH_VALIDATOR_ARGS > /dev/null