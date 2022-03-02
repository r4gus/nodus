# Nodus

A graphical __logic gate simulator__.

This project is in an early stage of development but most of the basic features are implemented.

## Features

Choose from a variety of logic gates, input controls and output controls to build digital circuits using
a drag and drop interface.

![Simple Circuit](images/example-app.png)

| Gates | Input controls | Output controls |
|:-----:|:--------------:|:---------------:|
|  AND  |   High const   |   Light bulb    |
| NAND |   Low const    |                 |
| OR | Toggle switch  |                 |
| NOR |     Clock      |                 |
| NOT |                |                 |
| XOR |                |                 |

![Logic Gate Selection](images/components.png)

Insert components into the world using a radial context menu.

![Context Menu](images/context.png)

Save the circuits you've created in a [.ron](https://github.com/ron-rs/ron) file and reload them later.

![Simple Circuit](images/save-load.png)

## Getting started

Setup the Rust development environment and Bevy.

### Installing Rust

Install Rust by following the [Getting Started Guide](https://www.rust-lang.org/learn/get-started).

### Setting up Bevy

Follow this [Guide](https://bevyengine.org/learn/book/getting-started/setup/) to setup Bevy.

### Run program

First clone the repository.
```
git clone https://github.com/r4gus/nodus.git
```

Then switch into the project folder and run the program.
```
cd nodus
cargo run
```

## Known Issues

Here are some tips to solve known issues.

### Ubuntu linker error

If you get the following link error in Ubunut/ Debian,

```
= note: /usr/bin/ld: cannot find -lxcb-render
          /usr/bin/ld: cannot find -lxcb-shape
          /usr/bin/ld: cannot find -lxcb-xfixes
          collect2: error: ld returned 1 exit status
```

try to install `libxcb-shape0-dev` and `libxcb-xfixes0-dev` separately, i.e. `sudo apt install libxcb-shape0-dev libxcb-xfixes0-dev`.

### AMD driver issue

If you get the following runtime error,

```
thread 'main' panicked at 'Failed to acquire next swap chain texture!: Timeout', /home/USERNAME/.cargo/registry/src/github.com-1ecc6299db9ec823/bevy_render-0.6.0/src/view/window.rs:161:24
```

you can either try to disable `vsync` in `src/main.rs`, or switch from `AMDVLK` to `RADV`: `AMD_VULKAN_ICD=RADV cargo run` (maybe [this](https://wiki.archlinux.org/title/Vulkan#Selecting_Vulkan_driver) can help).


## Controls

- `lmb pressed`: select/ drag (selection mode - `s`), pan (pan mode - `p`)
- `rmb pressed`: open context menu
- `mouse wheel`: zoom

## Planned Features

- [ ] Create new logic components from existing circuits
- [ ] More output controls (e.g. 7 segment display)
- [ ] Create truth tables from circuits

## Credits

* [The Bevy Engine](https://bevyengine.org/)
* [Bevy Prototype Lyon](https://github.com/Nilirad/bevy_prototype_lyon)
