#!/bin/bash

# Destination folder
PREFIX="$HOME/.config/polybar/scripts/ixwindow"

# Folder for cached icons
CACHE="$HOME/.config/polybar/scripts/ixwindow/polybar-icons"

# Folder for config icons
CONFIG_DIR="$HOME/.config/ixwindow"
CONFIG_FILE="$CONFIG_DIR/config.toml"

# Size of the icon
SIZE=24

# Background color of your polybar
COLOR="#252737"

# Window manager (current supported are bspwm and i3)
WM="bspwm"

# Coordinates of icon, you might wanna play around with
# GAP option in the ixwindow file as well
X=270
Y=6


generate_config () {
    local content="\
size = $SIZE
color = \"$COLOR\"\
"
    echo "$content" > "$CONFIG_FILE"
}


cp -R ixwindow ixwindow_compiled

mkdir -p "$CACHE"
mkdir -p "$PREFIX"
mkdir -p "$CONFIG_DIR"

generate_config


sed -i "s/\$X/$X/g" ixwindow_compiled/polybar-xwindow-icon.cpp
sed -i "s/\$Y/$Y/g" ixwindow_compiled/polybar-xwindow-icon.cpp
sed -i "s/\$SIZE/$SIZE/g" ixwindow_compiled/polybar-xwindow-icon.cpp


sed -i "s/\$\$SIZE/$SIZE/g" ixwindow_compiled/generate-icon
sed -i "s/\$\$COLOR/\"$COLOR\"/g" ixwindow_compiled/generate-icon

CACHE="$(echo "$CACHE" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"
DIR="$(echo "$PREFIX" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"
CONFIG_FILE="$(echo "$CONFIG_FILE" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"


sed -i "s/\$\$CONFIG/\"$CONFIG_FILE\"/g" ixwindow_compiled/ixwindow-convert

case "$WM"
    "bspwm")
        sed -i "s/\$\$CACHE/\"$CACHE\"/g" ixwindow_compiled/bspwm/ixwindow
        sed -i "s/\$\$DIR/\"$DIR\"/g" ixwindow_compiled/bspwm/ixwindow

        rm -r "ixwindow_compiled/i3"
        ;;
    "i3")
        sed -i "s/\$\$CACHE/\"$CACHE\"/g" ixwindow_compiled/i3/ixwindow
        sed -i "s/\$\$DIR/\"$DIR\"/g" ixwindow_compiled/i3/ixwindow
        
        rm -r "ixwindow_compiled/bspwm"
        ;;
esac

g++ ixwindow_compiled/polybar-xwindow-icon.cpp -o ixwindow_compiled/polybar-xwindow-icon -I/usr/include/opencv4/ -lopencv_core -lopencv_videoio -lopencv_highgui -lopencv_imgcodecs -lopencv_imgproc -lX11


mv ixwindow_compiled/* "$PREFIX"

rm -r ixwindow_compiled
