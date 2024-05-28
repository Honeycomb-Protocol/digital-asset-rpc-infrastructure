#!/usr/bin/env bash
source ~/.bashrc
docker compose down -v

sudo rm -r db-data > /dev/null && echo "db-data Flushed" || echo "db-data is empty"
sudo rm -r ledger > /dev/null && echo "ledger Flushed" || echo "ledger is empty"

docker compose up db redis -d
sleep 15s

docker compose up solana migrator -d
sleep 30s

docker compose up ingester api proxy -d

sleep 15s

solana airdrop 100 BkoJLLqagMPx5Dv2CpxwxEuTKR9DZQ85puq2zwKGdTJo -u http://localhost:7601
solana airdrop 100 EgukF3ucRRiTzTLQm2Wv9NXJGAXghMaDU7H2dfTfBHom -u http://localhost:7601
solana airdrop 100 FS6Bf3j4acWdiqvVKxbYN4rWZDotgqx7tcoL5jLsdR6P -u http://localhost:7601
solana airdrop 100 7G15ok4mrkf77tNP742R1sgG8W2pwDrcFnHs1C98U6XB -u http://localhost:7601
solana airdrop 100 2d2ZQ5AdxTDQdwsdrRD5o4PoD5cAj66WZx66JDAChamC -u http://localhost:7601
solana airdrop 100 6rQF92xvugYHMnrZmSuByR7e8JyWxh7MJbWiMQ2hwV59 -u http://localhost:7601
solana airdrop 100 2CqHuH8LEFeUGnv3xGGp2HJo6ED6aUZWvBd8k613piTj -u http://localhost:7601
solana airdrop 100 6knLqX76cM9acs5LFB6otAp8ux1mYupt2YVt5QjAhXrX -u http://localhost:7601


# solana airdrop 30 -k keys/admin.json
# solana airdrop 10 -k keys/driver.json
# solana airdrop 10 -k keys/user.json
# solana airdrop 10 -k key.json
# yarn ts-node scripts/createGlobal.ts
