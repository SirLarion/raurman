# raurman
Package management tool for Arch Linux combining pacman and AUR installations.
Packages can be saved into a JSON file by group for easy re-installations.
Written in Rust for cool points.

### Setup

```
git clone https://github.com/SirLarion/raurman
cd raurman
./install
```

### Usage

The interface is built to be pacman-like, so `-S` and `-R` will give you your
basic functionality.

To install something from the AUR, use the `-A`/`--aur` flag:

```
raurman -SA <AUR package>
```

To save or remove a package from the JSON, use the `-s`/`--save` flag. By
default the package is saved into all groups specified in the JSON, you can
define which group(s) to save in to with `-G`/`--group`:

```
raurman -S <AUR package> -sG <Group name>
```
