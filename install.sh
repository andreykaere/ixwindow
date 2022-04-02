#!/bin/bash

PREFIX="$HOME/.config/polybar/scripts/ixwindow"

SIZE=24
COLOR="#252737"
X=6
Y=207





g++ -o ixwindow/polybar-xwindow-icon ixwindow/polybar-xwindow-icon.cpp `pkg-config --cflags --libs opencv` -lX11



