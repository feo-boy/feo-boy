# feo-boy

[![Build Status](https://travis-ci.org/afishberg/feo-boy.svg?branch=master)](https://travis-ci.org/afishberg/feo-boy)

An emulator for the Nintendo Game Boy (DMG) written in Rust.

```sh
$ git clone https://github.com/afishberg/feo-boy && cd feo-boy
$ cargo run --release -- --bios path/to/bios.gb path/to/rom.gb
```

See all options with the `--help` flag.

```sh
$ cargo run --release -- --help
```

Enable log output by setting `RUST_LOG=feo_boy`.

## Debugging

Enable debug mode by passing the `--debug` flag. You will see a prompt that
allows you to step emulation an instruction at a time. You may also dump memory
or CPU state from this prompt.

Enter `?` at this prompt to see all debug options.

## Testing

Run unit tests with `cargo test`.
