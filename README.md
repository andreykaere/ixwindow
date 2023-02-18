# Ixwindow – iconized xwindow 

## About
Ixwindow is an enhanced version of standard `xwindow` polybar module. The main
feature is that `ixwindo` displays not only info about active window, but also 
an icon for it. It also allows you more customization of printing window info.
Below is represented and example of what `ixwindow` looks like in action:

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
- for `i3` you can additionally specify `gap_per_desk`. This variable is used
  for calculation position of the icon, when the number of active desktops is
  dynamic.

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

To uninstall, simply run `./uninstall <wm>`, where `<wm>` is the one you want
to uninstall files from. Make sure that paths, specified in the `uninstall` 
script, match the ones you actually use. If you want additionally to remove 
cached icons, you should run it with `--cache` option. For removing files for
all window managers run `./uninstall --all`.

## Configuration

To change your configuration, just edit your config file. For new settings to
take affect, you have to restart polybar (for example with `polybar-msg cmd
restart`).

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
ixwindow-convert --wm <wm> <icon-name>.png
```
where `<icon-name>` is the right name as described above and `<wm>` is the
name of window manager you want it to be generated for (i.e. the program will
use corresponding config file). For more info run `ixwindow-convert --help`.

**Note:** This method can be also used for replacing automatically generated
icons.

## Known issues & limitations

- Lack of png support, but replacing `jpg` with `png` would require compositor 
as the dependency as well
- Untested on multimonitors system
- Manual specification, but seems to be unfixable at this point, since polybar 
doesn't support inserting images into bar for now
- (for `bspwm`) Not being able to stop `./ixwindow` with `Ctrl-C`, due to background
  processes

Feel free to open issue if you have any questions or you've noticed a bug.
Also pull requests are welcome; don't hesitate to crate one, if you have a
solution to any of the issues, stated above.

## Goals

- Rewrite `polybar-xwindow-icon` in Rust
- Rewrite code for `bspwm` in Rust
- Add png support (maybe make it an option, if user doesn't use compositor)

## Thanks

### Inspired by

I got inspired to start this project, when I saw similar feature in
`awesome-wm`. I thought it would be hard to simulate the exact same behavior
on polybar, however I came across
[this](https://github.com/MateoNitro550/xxxwindowPolybarModule) project and I
thought, that I can just improve the default `xwindow`, by formatting its
output a bit and adding icon of the focused application.

### With a great help of

This project couldn't have been done without them:

- https://stackoverflow.com/questions/54513419/putting-image-into-a-window-in-x11
- https://unix.stackexchange.com/questions/48860/how-to-dump-the-icon-of-a-running-x-program

