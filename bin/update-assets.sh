#!/bin/bash

ROOT=$(dirname $0)/..

cd $ROOT/assets

echo "Generating VHS GIFs..."
vhs -q scaffolding-tui.tape > /dev/null

cd - > /dev/null
