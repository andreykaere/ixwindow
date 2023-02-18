# ixwindow – icon xwindow module for Polybar


## About

`ixwindow` is an enhanced version of standard `xwindow` polybar module. 
The main feature is icon for active window, but it also allows you more 
customization of printing window info. This is what `ixwindow` looks
like in action:

<p align="center">
  <img src="examples/example.gif" alt="animated" />
</p>


**Note:** basically, it doesn't really depend on polybar itself, it can be used 
with any other bar as well, you just need to implement the same behavior,
as polybar's `tail = true`.


## Dependencies

### Common 
- `xprop`
- `xdo`
- `imagemagick` (for converting `.png` icons to `.jpg`)
- `g++` (for compiling `polybar-xwindow-icon`)
- `opencv` (for rendering icons)

For debian-based systems you can install it by running
```bash
sudo apt install xdo xprop imagemagick g++ libopencv-dev 
```

### For bspwm
- `bash`
- `bspwm`
- `bspc`
- [`xdotool`](https://github.com/jordansissel/xdotool) (version 3.20211022.1)

For debian-based systems you can install it by running (make sure version is
correct, otherwise see note below):
```bash
sudo apt install xdotool
```

### For i3
- `i3`
- [`cargo`](https://github.com/rust-lang/cargo)

For cargo installation instructions, see [here](https://github.com/rust-lang/cargo).


**Note:** depending on your system, you might get different version of the
packages, comparing to the ones, used in this project. If you can't install
the right versions via built-in package manager (e.g. `apt`), then you will 
have to either build the newer version from source, or modify source code of 
this project to your versions. (if it's even possible)


## Installation

In directory `profiles` there are two templates for `bspwm` and `i3`. Modify
them if you need and then execute `./install <names of wm>` (e.g. to install
for both, you will have to run `./install "bspwm" "i3"`). 
Things to modify:
- background color of polybar bar
- size of icon
- coordinates for icon
- path to `polybar-icons` folder (note: it makes sense to keep it 
around `.config/polybar` folder, so you won't lose your custom icons, 
if you have them)
- `gap` constant, which is used in `ixwindow` script 

You will also need to add the following to your polybar `config` file:

```dosini
[module/ixwindow]
type = custom/script
exec = /path/to/ixwindow
tail = true
```

and put it somewhere on bar, for example, add it right next to `bspwm`: 
`modules-left = bspwm ixwindow`.

### Uninstallation

To uninstall, simply run `./uninstall`, but make sure that paths, specified in
the script, match the ones you use. If you want additionally to remove cached
icons, you should run it with `--cache` option.

## Generating icons

`ixwindow` uses the output of `xprop` for generating icons automatically. 
Most of the times it works, but for some applications, for example, Spotify,
it doesn't work. Then, if you want to have an icon for these applications, you 
have to add them yourself. 

### Adding custom icons

Sometimes it's not possible to get icon using `xprop`, (for example, it's the 
case with Spotify and Discord), then you have to add them manually to your 
`polybar-icons` folder. To do that, you need to have `.png` version of the 
icon, named as `WM_CLASS` (you can find it by running `xprop WM_CLASS` and 
selecting your app). Then you run the following command (requires `imagemagick`): 
```bash
ixwindow-convert <icon-name>.png
```
where `<icon-name>` is the right name as described above.

**Note:** This method can be also used for replacing automatically generated
icons.

## Known issues & limitations

- Lack of png support, but replacing `jpg` with `png` would require compositor 
as the dependency as well
- Untested on multimonitors system
- Manual specification, but seems to be unfixable at this point, since polybar 
doesn't support inserting images into bar for now
- Not being able to stop `./ixwindow` with `Ctrl-C`, due to background
  processes

Feel free to open issue if you have any questions or you've noticed a bug.
Also pull requests are welcome; don't hesitate to crate one, if you have a
solution to any of the issues, stated above.

## Thanks

### Inspired by  

https://github.com/MateoNitro550/xxxwindowPolybarModule

### With a great help of

- https://stackoverflow.com/questions/54513419/putting-image-into-a-window-in-x11
- https://unix.stackexchange.com/questions/48860/how-to-dump-the-icon-of-a-running-x-program

