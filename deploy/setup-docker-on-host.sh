#!/bin/bash

set -e

echo "Logging in..."
docker login ghcr.io -u USERNAME -p $1

docker pull ghcr.io/econaxis/test:latest

echo "Killing existing containers if exist"
docker kill main &> /dev/null  || :
docker rm main &> /dev/null  || :
sleep 0.2

echo "Running new container"
docker run --name main -d -p 443:3030 --restart=on-failure:10 -v $HOME/data2:/app:rw -e RUST_LOG=info,timetoreach=debug,h2=info,hyper=info,warp=info,rustls=info ghcr.io/econaxis/test:latest

docker logs main -f