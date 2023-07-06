#!/bin/sh

set -e
source deploy/.env.prod
docker build -t ghcr.io/econaxis/test . && docker push ghcr.io/econaxis/test

echo $GITHUB_PAT
ssh 35.238.190.254 -t "sh setup-docker-on-host.sh $GITHUB_PAT; exec bash -l"