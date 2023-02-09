#!/bin/bash

# Destination folder
PREFIX="$HOME/.config/polybar/scripts/ixwindow"

# Folder for cached icons
CACHE="$HOME/.config/polybar/scripts/ixwindow/polybar-icons"

# Folder for config icons
CONFIG_PREFIX="$HOME/.config/ixwindow"

# Size of the icon
SIZE=24

# Background color of your polybar
COLOR="#252737"

# Coordinates of icon, you might wanna play around with
# GAP option in the ixwindow file as well
X=270
Y=6


generate_config () {
    local wm="$1"
    local config_file="$CONFIG_PREFIX/$wm/config.toml"
    local content="\
size = $SIZE
color = \"$COLOR\"\
"
    echo "$content" > "$config_file"
}


cp -R ixwindow ixwindow_compiled

mkdir -p "$CACHE"
mkdir -p "$PREFIX"
mkdir -p "$CONFIG_PREFIX/bspwm"

generate_config "bspwm"


sed -i "s/\$X/$X/g" ixwindow_compiled/polybar-xwindow-icon.cpp
sed -i "s/\$Y/$Y/g" ixwindow_compiled/polybar-xwindow-icon.cpp
sed -i "s/\$SIZE/$SIZE/g" ixwindow_compiled/polybar-xwindow-icon.cpp


sed -i "s/\$\$SIZE/$SIZE/g" ixwindow_compiled/generate-icon
sed -i "s/\$\$COLOR/\"$COLOR\"/g" ixwindow_compiled/generate-icon

CACHE="$(echo "$CACHE" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"
DIR="$(echo "$PREFIX" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"
CONFIG_DIR="$(echo "$CONFIG_PREFIX" | sed -e 's/\\/\\\\/g; s/\//\\\//g; s/&/\\\&/g')"


sed -i "s/\$\$CACHE/\"$CACHE\"/g" ixwindow_compiled/ixwindow
sed -i "s/\$\$DIR/\"$DIR\"/g" ixwindow_compiled/ixwindow
sed -i "s/\$\$CONFIG_DIR/\"$CONFIG_DIR\"/g" ixwindow_compiled/ixwindow-convert


g++ ixwindow_compiled/polybar-xwindow-icon.cpp -o ixwindow_compiled/polybar-xwindow-icon -I/usr/include/opencv4/ -lopencv_core -lopencv_videoio -lopencv_highgui -lopencv_imgcodecs -lopencv_imgproc -lX11


mv ixwindow_compiled/* "$PREFIX"

rm -r ixwindow_compiled
