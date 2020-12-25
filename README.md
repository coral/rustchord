# Rustchord (Colorchord2 bindings for Rust)

These are simple bindings around the Notefinder construct in [Colorchord](https://github.com/cnlohr/colorchord). Colorchord is an amazing piece of software written by [CNLohr](https://github.com/cnlohr). I've been wanting to use the algorithm in different projects and decided to write a easy Rust bindings that allows you to run the algorithm against audio acquired in Rust.

### Building from Git

```
git clone https://github.com/coral/rustchord.git
cd rustchord
git submodule update --init --recursive
cargo build
```

To run the audio input example, just do
`cargo run --example audioinput`

### License

ColorChord is Copyright 2015 Charles Lohr, Under the MIT/x11 License.

All other code is licensed under the MIT license.
