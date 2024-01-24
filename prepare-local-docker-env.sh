#!/bin/bash

# output colours
RED() { echo $'\e[1;31m'$1$'\e[0m'; }
GRN() { echo $'\e[1;32m'$1$'\e[0m'; }

CURRENT_DIR=$(pwd)
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
# go to parent folder
cd $(dirname $(dirname $SCRIPT_DIR))

EXTERNAL_ID=("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s" \
"BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY" \
"cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK" \
"noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV" \
"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" \
"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" \
"TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb" \
)
EXTERNAL_SO=("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s.so" \
"BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY.so" \
"cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK.so" \
"noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV.so"
"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL.so" \
"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA.so" \
"TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb.so" \
)

mkdir solana_program_library || true
curl -LkSs https://api.github.com/repos/solana-labs/solana-program-library/tarball/token-swap-js-v0.4.0 | tar -xz --strip-components=1 -C ./solana_program_library
tar -zxf -C /solana_program_library solana-program-library.tar.gz
pushd solana_program_library/account-compression/programs/account-compression
  cargo build-bpf --bpf-out-dir ./here
  mv ./here/spl_account_compression.so $CWD/programs/cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK.so
popd

if [ -z "$OUTPUT" ]; then
    OUTPUT=$CURRENT_DIR/programs
fi

pushd solana_program_library/associated-token-account/program
  cargo build-bpf --bpf-out-dir ./here
  mv ./here/spl_associated_token_account.so $CWD/programs/ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL.so
popd

# dump external programs binaries if needed
for i in ${!EXTERNAL_ID[@]}; do
    if [ ! -f "${OUTPUT}/${EXTERNAL_SO[$i]}" ]; then
        solana program dump -u $RPC ${EXTERNAL_ID[$i]} ${OUTPUT}/${EXTERNAL_SO[$i]}
    else
        solana program dump -u $RPC ${EXTERNAL_ID[$i]} ${OUTPUT}/onchain-${EXTERNAL_SO[$i]} > /dev/null
        ON_CHAIN=`sha256sum -b ${OUTPUT}/onchain-${EXTERNAL_SO[$i]} | cut -d ' ' -f 1`
        LOCAL=`sha256sum -b ${OUTPUT}/${EXTERNAL_SO[$i]} | cut -d ' ' -f 1`

        if [ "$ON_CHAIN" != "$LOCAL" ]; then
            echo $(RED "[ WARNING ] on-chain and local binaries are different for '${EXTERNAL_SO[$i]}'")
        else
            echo "$(GRN "[ SKIPPED ]") on-chain and local binaries are the same for '${EXTERNAL_SO[$i]}'"
        fi

rm -rf solana_program_library
rm -rf metaplex_program_library
