#!/bin/sh

if [[ -z REMOTE_BUILD ]]; then
  echo "Remote build"
else
  echo "Remote build false"
  source deploy/.env.prod
fi

docker login ghcr.io -u USERNAME -p GITHUB_PAT
docker build -t ghcr.io/econaxis/test .
docker push ghcr.io/econaxis/test
if [[ -z REMOTE_BUILD ]]; then
  sh ~/data2/deploy/setup-docker-on-host.sh $GITHUB_PAT
else
  rsync -rvahz --progress --relative ./city-gtfs/ ./web/public/ ./deploy/ ./vancouver-cache/ ./certificates/ 35.238.190.254:data2/
  sleep 0.1
  ssh -o StrictHostKeyChecking=no 35.238.190.254 -t "pwd; sh data2/deploy/setup-docker-on-host.sh $GITHUB_PAT; exec bash -l"
fi

