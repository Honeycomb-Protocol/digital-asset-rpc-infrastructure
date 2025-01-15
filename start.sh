#!/usr/bin/env bash
docker compose build
docker compose build
docker compose build
source ~/.bashrc

docker compose up db redis -d
sleep 10s

docker compose up migrator -d
sleep 10s

docker compose up ingester api proxy -d
sleep 10s

docker compose up -d
docker compose down transactions-backfiller accounts-backfiller
