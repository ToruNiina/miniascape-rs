# miniascape

[![dependency status](https://deps.rs/repo/github/emilk/eframe_template/status.svg)](https://deps.rs/repo/github/emilk/eframe_template)
[![Build Status](https://github.com/emilk/eframe_template/workflows/CI/badge.svg)](https://github.com/emilk/eframe_template/actions?workflow=CI)

Run your cellular automaton.

[open miniascape](https://toruniina.github.io/miniascape-rs/)

This repo is based on [eframe_template](https://github.com/emilk/eframe_template).

## What is this?

`miniascape` provides you an environement to run a cellular automaton with a rule you defined.

It utilizes [rhai](https://rhai.rs) as a scripting language to describe a rule.

`rhai` script you wrote will be evaluated in each cell while the simulation.

You can use arbitrary type as a cell state, including `bool`, `i32`, `f32`, `rhai::Array`, etc.

You can use not only square grid with Moore Neighborhood, but also a hex grid and a square grid with Von Neumann neighborhood.

## Build

### Compiling for the web

Make sure you are using the latest version of stable rust by running `rustup update`.

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page. For this you need to set up some tools. There are a few simple scripts that help you with this:

```sh
./setup_web.sh
./build_web.sh
./start_server.sh
open http://127.0.0.1:8080/
```

### Testing locally

**NOTE** First you need to comment out `"wasm-bindgen"` feature of `rhai` in `Cargo.toml` when you run it natively.

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel`

For running the `build_web.sh` script you also need to install `jq` and `binaryen` with your packet manager of choice.

