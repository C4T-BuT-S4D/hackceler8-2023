#!/bin/sh
set -e

python3 -m venv env
source env/bin/activate
python3 -m pip install -r game/requirements.txt

cd ../cheats-rust
maturin build --profile opt
pip install target/wheels/*
cd ../game


certdir=$(dirname "$2")
certdir="`cd "${certdir}";pwd`"
certfile=$(basename "$2")

cd game
python3 client.py "$1" 8888 8889 "$certdir/$certfile"
