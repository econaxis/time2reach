#!/bin/bash

set -e

echo "Logging in..."
docker login ghcr.io -u USERNAME -p $1

docker pull ghcr.io/econaxis/test:latest

echo "Killing existing containers if exist"
docker kill main &> /dev/null  || :
docker rm main &> /dev/null  || :
sleep 1

echo "Running new container"
docker run --rm --name main -p 443:3030 --log-driver=gcplogs ghcr.io/econaxis/test:latest

docker logs main -f