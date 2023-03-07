# Ixwindow – iconized xwindow 

## About
Ixwindow is an enhanced version of standard `xwindow` polybar module. The main
feature is that `ixwindow` displays not only info about active window, but also 
an icon for it. It also allows you more customization of printing window info.
Below is represented an example of what `ixwindow` looks like in action:

<p align="center">
  <img src="assets/bspwm_example.gif" alt="animated" />
</p>

**Note:** basically, it doesn't really depend on polybar itself, it can be used 
with any other bar as well, you just need to implement the same behavior,
as polybar's `tail = true`.


## Dependencies

### Common 
- `xprop`
- `imagemagick` (for converting `.png` icons to `.jpg`)
- [`cargo`](https://github.com/rust-lang/cargo)

For debian-based systems you can install it by running
```bash
sudo apt install xprop imagemagick
```
For cargo installation instructions, see [here](https://github.com/rust-lang/cargo).

### For bspwm
- `bash`
- `bspwm`
- `bspc`
- `xdo`
- [`xdotool`](https://github.com/jordansissel/xdotool) (version 3.20211022.1)

For debian-based systems you can install it by running (make sure version is
correct, otherwise see note below):
```bash
sudo apt install xdotool xdo
```

### For i3
- `i3`

**Note:** depending on your system, you might get different version of the
packages, comparing to the ones, used in this project. If you can't install
the right versions via built-in package manager (e.g. `apt`), then you will 
have to either build the newer version from source, or modify source code of 
this project to your versions. (if it's even possible)


## Downloading

If you want to install stable version, then you should download the source code 
from the `master` branch using the following command:
```bash
git clone git@github.com:andreykaere/ixwindow.git && cd ixwindow
```
If you want the bleeding edge version, switch to `dev` branch.

### Installation

Execute `./install <names of wm>` (e.g. to install for both, you can run
`./install "bspwm" "i3"`). To see more options run `./install --help`. 
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

Default configuration file is supposed to be located at
`$XDG_CONFIG_HOME/ixwindow/ixwindow.toml`. If you want it to be located
somewhere else, you should specify that in environmental variable
`IXWINDOW_CONFIG_PATH`, or run `ixwindow` script with
`--config=<path_to_config>` option.

In config file, there are various options, that can be modified (example of
configuration file can be found in `examples/ixwindow.toml`), such as:
- background color of polybar bar
- size of icon
- coordinates for icon
- path to folder for cached icons (note: it makes sense to keep it 
around `.config/polybar` folder, so you won't lose your custom icons, 
if you have them)
- `gap` constant, which is used in `ixwindow` script 
- for `i3` you can additionally specify `gap_per_desk`. This variable is used
  for calculation position of the icon, when the number of active desktops is
  dynamic.

To change your configuration, just edit your config file. For new settings to
take affect, you have to restart polybar (for example with `polybar-msg cmd
restart`).

## Generating icons

`ixwindow` uses the output of `xprop` for generating icons automatically. 
Most of the times it works, but for some applications, for example, Spotify,
it doesn't. In this case, if you want to have an icon for these applications,
you have to add them manually. 

### Adding custom icons

Sometimes it's not possible to get icon using `xprop`, (for example, it's the 
case with Spotify and Discord), then you have to add them manually to your 
`polybar-icons` folder. To do that, you need to have `png` or `svg` version
of the icon, named as `WM_CLASS` (which you can find by running `xprop
WM_CLASS` and selecting your app). Then you run the following command
(requires `imagemagick`): 
```bash
ixwindow-convert --size <size> --color <color> --cache <chache_dir> <icon-name>
```
where `<icon-name>` is the right name as described above. This will convert
icon to `jpg` format with appropriate background color and move to your cache 
directory. 

**Note:** This method can be also used for replacing automatically generated
icons. 

**Note:** Basically all apps have icons on your system in `png` or `svg`
format. Usually, one can find it somewhere in `/usr/share/icons` directory
(one can use `find` or `fd` utility for it).

You can try it out on some icons located in `examples/custom-icons` folder.

## Known issues & limitations

- Lack of png support, but replacing `jpg` with `png` would require compositor 
as the dependency as well
- Manual specification, but seems to be unfixable at this point, since polybar 
doesn't support inserting images into bar for now
- (for `bspwm`) Not being able to stop `./ixwindow` with `Ctrl-C`, due to background
  processes

Feel free to open issue if you have any questions or you've noticed a bug.
Also pull requests are welcome; don't hesitate to crate one, if you have a
solution to any of the issues, stated above.

## Goals

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

I would like to thank:
- [psychon](https://github.com/psychon) for helping me understand `x11rb` and
xorg in general
- [This
  link](https://unix.stackexchange.com/questions/48860/how-to-dump-the-icon-of-a-running-x-program),
  which code is used for automatic icon generation

