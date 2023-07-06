#!/bin/sh

set -e
source deploy/.env.prod


docker login ghcr.io -u USERNAME -p $DOCKER_WRITE

docker build -t ghcr.io/econaxis/test . && docker push ghcr.io/econaxis/test

ssh 35.238.190.254 -t "sh setup-docker-on-host.sh $GITHUB_PAT; exec bash -l"