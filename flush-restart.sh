#!/usr/bin/env bash
source ~/.bashrc
docker compose down -v --remove-orphans

sudo rm -r /mnt/storage/localnet > /dev/null && echo "/mnt/storage/localnet Flushed" || echo "/mnt/storage/localnet is empty"
# sudo rm -r ledger > /dev/null && echo "ledger Flushed" || echo "ledger is empty"

docker compose up solana db redis -d
sleep 5s

docker compose up migrator -d
sleep 10s

docker compose up -d

sleep 5s

solana airdrop 1000 BkoJLLqagMPx5Dv2CpxwxEuTKR9DZQ85puq2zwKGdTJo -u http://localhost:9098
solana airdrop 1000 EgukF3ucRRiTzTLQm2Wv9NXJGAXghMaDU7H2dfTfBHom -u http://localhost:9098
solana airdrop 1000 7woDY25ZEYi24RNnuUbvsdZvf9dFgsC6ZLUd2vu3j9pn -u http://localhost:9098
solana airdrop 1000 FS6Bf3j4acWdiqvVKxbYN4rWZDotgqx7tcoL5jLsdR6P -u http://localhost:9098
solana airdrop 1000 7G15ok4mrkf77tNP742R1sgG8W2pwDrcFnHs1C98U6XB -u http://localhost:9098
solana airdrop 1000 2d2ZQ5AdxTDQdwsdrRD5o4PoD5cAj66WZx66JDAChamC -u http://localhost:9098
solana airdrop 1000 6rQF92xvugYHMnrZmSuByR7e8JyWxh7MJbWiMQ2hwV59 -u http://localhost:9098
solana airdrop 1000 2CqHuH8LEFeUGnv3xGGp2HJo6ED6aUZWvBd8k613piTj -u http://localhost:9098
solana airdrop 1000 6knLqX76cM9acs5LFB6otAp8ux1mYupt2YVt5QjAhXrX -u http://localhost:9098


# solana airdrop 30 -k keys/admin.json
# solana airdrop 10 -k keys/driver.json
# solana airdrop 10 -k keys/user.json
# solana airdrop 10 -k key.json
# yarn ts-node scripts/createGlobal.ts

# docker compose logs transactions-backfiller accounts-backfiller -f