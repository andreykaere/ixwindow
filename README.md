# ixwindow – icon xwindow module for Polybar


## About

`ixwindow` is an enhanced version of standard `xwindow` polybar module. The main feature is icon for active window, but it also allows you more customization of printing window info. `ixwindow` in work:

<p align="center">
  <img src="example.gif" alt="animated" />
</p>

## Installation

Just modify `install.sh` script for your case and run it. Things to specify:
- background color of polybar bar
- size of icon
- coordinates for icon
- path to `ixwindow` folder (default: `$HOME/.config/polybar/scripts/ixwindow`)

You will also need to add this to your polybar `config` file:

```
[module/ixwindow]
type = custom/script
exec = /path/to/ixwindow
tail = true
```



## Known issues & limitations

- Unable to kill the previous ixwindow, without addining it to path
- Add png support, but it will require compositor as well
- Untested on multi monitors system

## Thanks

### Inspired by  

https://github.com/MateoNitro550/xxxwindowPolybarModule

### With great help of

https://stackoverflow.com/questions/54513419/putting-image-into-a-window-in-x11

