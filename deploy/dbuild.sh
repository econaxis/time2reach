#!/bin/sh

source deploy/.env.prod
#set -e


docker login ghcr.io -u USERNAME -p $DOCKER_WRITE

docker build -t ghcr.io/econaxis/test .
docker push ghcr.io/econaxis/test

rsync -rvahz --progress --relative ./city-gtfs/ ./web/public/ ./deploy/ ./certificates/ 35.238.190.254:data2/

sleep 0.1

ssh -o StrictHostKeyChecking=no 35.238.190.254 -t "pwd; sh data2/deploy/setup-docker-on-host.sh $GITHUB_PAT; exec bash -l"