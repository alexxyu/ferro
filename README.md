# Ferro
[![Run ferro tests and coverage](https://github.com/alexxyu/ferro/actions/workflows/ci.yml/badge.svg)](https://github.com/alexxyu/ferro/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/alexxyu/ferro/branch/main/graph/badge.svg?token=4PG6RZJW56)](https://codecov.io/gh/alexxyu/ferro)

A lightweight text editor built in Rust.

Based on [Philipp Flenker's](https://www.philippflenker.com/hecto/) Rust text editor tutorial.

## Features

* vim-like navigation controls
* Syntax highlighting for Rust, Java, and Python (with more languages coming soon)
* Incremental forward and backward search
* Search-and-delete / search-and-replace
* Auto-indentation
* Built-in calculator for math expressions

## Documentation

API documentation is located [here](https://alexxyu.github.io/ferro/).

For reference on controls and usage, see [`docs/usage.md`](https://github.com/alexxyu/ferro/blob/main/docs/usage.md).

If you're interested in contributing, check out [`CONTRIBUTING.md`](https://github.com/alexxyu/ferro/blob/main/CONTRIBUTING.md).

## Installation

### Using `cargo`

The simplest way to install `ferro` is to run `cargo install ferro` on your machine. You can then
run `ferro [file]` to start editing!

### From source

The following instructions assume that you've already [installed Rust](https://www.rust-lang.org/tools/install).

```
git clone https://github.com/alexxyu/ferro
cd ferro
cargo build --release
```

This will generate the binary for `ferro` in the `target/release` directory.
