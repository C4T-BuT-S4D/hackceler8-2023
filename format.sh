#!/bin/sh
set -e

curdir=$(pwd)

cd game/game

# autoflake --recursive --in-place --remove-all-unused-imports .
isort --force-single-line-imports .
black .

cd $curdir
