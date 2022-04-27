# Mumblelink for Linux

This repo contains an .so file, for use with `Solar Tweaks`
(https://github.com/Solar-Tweaks/).

## Dependencies
- Lunar Client
- A rust compiler (to install, visit https://rustup.rs)

## First time setup
Before first launch you need to launch Lunar Client through the official
launcher.

### Setting up the repo
First clone this repo:
```bash
$ git clone https://github.com/dtomvan/mumblelink-on-linux
$ cd mumblelink-on-linux
```

### Compiling the native
Run:
```bash
$ cd ../..
$ cargo b --release
$ cp target/release/libMumbleLink.so SolarPatcher
```

### Downloading Solar Tweaks
If you are on Arch linux, type in the following commands:
```bash
$ git clone https://aur.archlinux.org/solar-tweaks-bin.git
$ cd solar-tweaks-bin
$ makepkg -si
```

Otherwise, go to https://github.com/Solar-Tweaks/Solar-Tweaks/releases and
download the latest AppImage.

Make it executable with:
```bash
$ chmod +x <AppImage>
```

### Launching Solar Tweaks
If you are on Arch Linux, you can use your program launcher (if you have one) to
launch `Solar Tweaks`. There is also the command `solartweaks`.

If you are not on Arch Linux, find where you have downloaded the AppImage and
launch it:
```bash
$ ./path/to/Solar-Tweaks.AppImage
```

### Configuring Solar Tweaks
In Solar Tweaks, click the `Patcher` tab, and enable `Mumble Fix`.
Make sure that in the `Settings` tab, the `Skip Checks` checkbox is **NOT**
ticked.

### Setting up the native file
For each version of Minecraft you want to run with Mumble Link, run (back in the
mumblelink-on-linux folder):
```bash
$ cp path/to/target/release/libMumbleLink.so ~/.lunarclient/offline/<version>/natives
```
