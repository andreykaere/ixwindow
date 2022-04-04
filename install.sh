#!/bin/bash

# Destination folder
PREFIX="$HOME/.config/polybar/scripts/ixwindow"

SIZE=24
COLOR="#252737"
X=270
Y=6

CACHE='$HOME/.config/polybar/scripts/ixwindow/polybar-icons'
DIR='$HOME/.config/polybar/scripts/ixwindow'


cp -R ixwindow ixwindow_compiled

# If no "$CACHE", create directory for icons
if ! [ -d "$CACHE" ]; then
    mkdir -p ixwindow_compiled/polybar-icons
fi



sed -i "s/\$X/$X/g" ixwindow_compiled/polybar-xwindow-icon.cpp
sed -i "s/\$Y/$Y/g" ixwindow_compiled/polybar-xwindow-icon.cpp
sed -i "s/\$SIZE/$SIZE/g" ixwindow_compiled/polybar-xwindow-icon.cpp


sed -i "s/\$\$SIZE/$SIZE/g" ixwindow_compiled/generate-icon
sed -i "s/\$\$COLOR/\"$COLOR\"/g" ixwindow_compiled/generate-icon

CACHE="$(echo "$CACHE" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"
DIR="$(echo "$DIR" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"


sed -i "s/\$\$CACHE/\"$CACHE\"/g" ixwindow_compiled/ixwindow
sed -i "s/\$\$DIR/\"$DIR\"/g" ixwindow_compiled/ixwindow


g++ -o ixwindow_compiled/polybar-xwindow-icon ixwindow_compiled/polybar-xwindow-icon.cpp `pkg-config --cflags --libs opencv` -lX11



rm -r "$PREFIX"

cp -R ixwindow_compiled "$PREFIX"

rm -r ixwindow_compiled
