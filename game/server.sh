#!/bin/sh
set -e

python3 -m venv env-server
source env-server/bin/activate
python3 -m pip install -r game/requirements.txt

if [ "$1" = "plain" ]; then
  export GAME_SSL="f"
elif [ "$1" = "ssl" ]; then
  export GAME_SSL="t"
else
  echo "Specify plain/ssl as first argument"
  exit 1
fi

cd game
python3 server.py --hostname=localhost 8888 team6 ../ca/team6
