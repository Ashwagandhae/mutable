# Mutable

An evolution simulator written in Rust. Currently proof of concept - muscle movement is based on simple sine waves, not brain activity, organisms can't sense their surroundings, and breeding is unimplemented. However, the evolution of plants, detritivores, and sometimes herbivores has already been observed.

Inspired by and based on:

- [The Bibites](https://thebibites.itch.io/the-bibites)
- [GenePool](https://www.swimbots.com/)
- [The Life Engine](https://thelifeengine.net/)
- [Cephalopods](https://github.com/jobtalle/cephalopods)

## Usage

```bash
git clone https://github.com/Ashwagandhae/mutable.git
cd mutable
cargo run --release
```

Note: I hope to support WebAssembly at some point, but for now you'll need to run it locally.

## Why?

Plenty of evolution simulators exist, but most make an arbitrary distinction between plants and animals, usually manifesting in circles of "food" spawning around the world for the evolving animals to eat. I wanted to make a simulator where the distinction between plants and animals is blurred, allowing both to evolve both together in a single world.

## Prerequisites

### Windows

1. Download [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
2. Install [Rust](https://www.rust-lang.org/tools/install).

### MacOS

1. Install CLang and macOS Development Dependencies.

```bash
xcode-select --install
```

2. Install [Rust](https://www.rust-lang.org/tools/install).

### Linux

1. Install a C compiler, depending on the distro.
2. Install [Rust](https://www.rust-lang.org/tools/install).
