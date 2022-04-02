# ixwindow – icon xwindow module for Polybar


## About

`ixwindow` is an enhanced version of standard `xwindow` polybar module. 
The main feature is icon for active window, but it also allows you more 
customization of printing window info. `ixwindow` in work:

<p align="center">
  <img src="example.gif" alt="animated" />
</p>


**Note:** basically, it doesn't depent on polybar one bit, it can be used 
with any other bar as well, you just need to implement the same behavior,
as polybar's `tail = true`.

## Installation

Just modify `install.sh` script for your case and run it. Things to modify:
- background color of polybar bar
- size of icon
- coordinates for icon
- path to `ixwindow` folder (default: `$HOME/.config/polybar/scripts/ixwindow`)
- path to `polybar-icons` folder (note: it makes sense to keep it 
around `.config/polybar` folder, so you won't lose your custom icons, 
if you have them)
- change `GAP` constant in `ixwindow` script 

You will also need to add this to your polybar `config` file:

```dosini
[module/ixwindow]
type = custom/script
exec = /path/to/ixwindow
tail = true
```

## Adding custom icons

Sometimes it's not possible to get icon using `xprop`, for example, it's the case with Spotify, 
so you might want to add them manually to your `polybar-icons` folder. To do that, you need to 
have `.png` version of icon and run, for example, the following commands (requires `imagemagick`):
```bash
convert Spotify.png -resize 24x24 -background "#252737" -flatten -alpha off Spotify.jpg
```

## Known issues & limitations

- Unable to kill the previous ixwindow, without addining it to path
- Add png support, but it will require compositor as well
- Untested on multimonitors system
- Manual specification, but seems to be unfixable at this point
- Works only with bspwm 

## Thanks

### Inspired by  

https://github.com/MateoNitro550/xxxwindowPolybarModule

### With great help of

https://stackoverflow.com/questions/54513419/putting-image-into-a-window-in-x11

