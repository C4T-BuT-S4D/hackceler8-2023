#!/bin/sh
set -e

python3 -m venv env
source env/bin/activate
python3 -m pip install -r game/requirements.txt

cd ../cheats-rust
maturin build --profile opt
pip install target/wheels/*
cd ../game

if [ "$1" = "local" ]; then
  hostname="localhost"
  certpath="../ca/team6"
  capath="../ca/CA-devel.crt"
  standalone=""
  export GAME_LOCAL="t"
elif [ "$1" = "remote" ]; then
  hostname="team6.hackceler8-2023.ctfcompetition.com"
  certpath="$HOME/team6"
  capath="$HOME/CA.crt"
  standalone=""
elif [ "$1" = "standalone" ]; then
  hostname=""
  certpath=""
  capath=""
  standalone="--standalone"
  export GAME_LOCAL="t"
else
  echo "Specify local/remote/standalone as first argument"
  exit 1
fi

if [ "$2" = "plain" ]; then
  export GAME_SSL="t"
elif [ "$2" = "ssl" ]; then
  export GAME_SSL="f"
else
  echo "Specify plain/ssl as second argument"
  exit 1
fi

cd game
python3 client.py --ca="$capath" $standalone "$hostname" 8888 8889 "$certpath"
