#!/bin/bash

source parse_toml


PROFILE="$(cat "$1")"


cp -R ixwindow ixwindow_compiled

mkdir -p "$CACHE"
mkdir -p "$PREFIX"
mkdir -p "$CONFIG_DIR"

generate_config


CONFIG_FILE="$(echo "$CONFIG_FILE" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"


sed -i "s/\$\$CONFIG/\"$CONFIG_FILE\"/g" ixwindow_compiled/ixwindow-convert

case "$WM"
    "bspwm")

        rm -r "ixwindow_compiled/i3"
        ;;
    "i3")
        
        rm -r "ixwindow_compiled/bspwm"
        ;;
esac

g++ ixwindow_compiled/polybar-xwindow-icon.cpp -o ixwindow_compiled/polybar-xwindow-icon -I/usr/include/opencv4/ -lopencv_core -lopencv_videoio -lopencv_highgui -lopencv_imgcodecs -lopencv_imgproc -lX11


mv ixwindow_compiled/* "$PREFIX"

rm -r ixwindow_compiled
