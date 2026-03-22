# Baloogah

It updates all runing docker images

## Why

You shouldn't use this. I wanted to learn Rust and TUIs. I then got bored and
vibed up the TUI part.  It does however fit my usecase.

## Release

```shell
cargo build -r
cp target/release/baloogah ~/bin
```

## Release aarch64

```shell
cross build --release
```
