#!/bin/sh
set -e

python3 -m venv server-env
source server-env/bin/activate
python3 -m pip install -r game/requirements.txt

cd game
python3 server.py --hostname=localhost 8888 team6 ../ca/team6
