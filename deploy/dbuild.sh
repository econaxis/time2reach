#!/bin/sh

HOST=34.173.85.101

if [[ -z REMOTE_BUILD ]]; then
  echo "Remote build"
else
  echo "Remote build false"
  source deploy/.env.prod
fi

docker login ghcr.io -u USERNAME -p $DOCKER_WRITE
docker build -t ghcr.io/econaxis/test .
docker push ghcr.io/econaxis/test

#rsync -rvahz --progress --relative --checksum ./city-gtfs/ ./web/public/ ./deploy/ ./vancouver-cache/ ./certificates/ california-big.db $HOST:data2/
sleep 0.1
ssh -o StrictHostKeyChecking=no $HOST -t "pwd; sh data2/deploy/setup-docker-on-host.sh $GITHUB_PAT; exec bash -l"
