#!/bin/bash

ROOT=$(dirname $0)/..

cd $ROOT/assets

echo "Generating VHS GIFs..."
vhs -q todo.tape > /dev/null & vhs -q shiny-tui.tape > /dev/null

cd - > /dev/null
