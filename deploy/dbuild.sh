#!/bin/sh

source deploy/.env.prod
#set -e


if [[ -z $GITHUB_ACTIONS ]]; then
  docker login ghcr.io -u USERNAME -p $DOCKER_WRITE
  rsync -rvahz --progress --relative ./city-gtfs/ ./web/public/ ./deploy/ ./certificates/ 35.238.190.254:data2/
  docker build -t ghcr.io/econaxis/test .
  docker push ghcr.io/econaxis/test

fi

sleep 0.1

ssh -o StrictHostKeyChecking=no 35.238.190.254 -t "pwd; sh data2/deploy/setup-docker-on-host.sh $GITHUB_PAT; exec bash -l"