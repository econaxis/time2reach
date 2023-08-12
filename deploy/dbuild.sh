#!/bin/sh

HOST=time2reach.duckdns.org

if [[ -z REMOTE_BUILD ]]; then
  echo "Remote build"
else
  echo "Remote build false"
  source deploy/.env.prod
fi

docker login ghcr.io -u USERNAME -p $DOCKER_WRITE
docker build -t ghcr.io/econaxis/test .
docker push ghcr.io/econaxis/test

rsync -rvahz --progress --relative --checksum ./city-gtfs/ ./web/public/ ./deploy/ ./vancouver-cache/ ./certificates/ $HOST:data2/
sleep 0.1
ssh -o StrictHostKeyChecking=no $HOST -t "pwd; sh data2/deploy/setup-docker-on-host.sh $GITHUB_PAT; exec bash -l"



#rsync -rvahzP --filter=":- target :- .git" --exclude="web/dist" --exclude=".git" --exclude="cache" --exclude="target" --exclude="venv*" --exclude="web/node_modules" . 34.30.48.109:data2
