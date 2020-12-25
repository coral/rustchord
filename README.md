# Rustchord (Colorchord2 bindings for Rust)

These are simple bindings around the Notefinder construct in [Colorchord](https://github.com/cnlohr/colorchord). Colorchord is an amazing piece of software written by [CNLohr](https://github.com/cnlohr). I've been wanting to use the algorithm in different projects and decided to write a easy Rust bindings that allows you to run the algorithm against audio acquired in Rust.

### Using the binding

The Notefinder expects to read samples from a ringbuffer in order to generate the bucketed notes. For this reason you need to provide a simple ringbuffer with audio samples in `f32` format. The `audioinput` showcases how to do this.

First create a new instance of the Notefinder with the samplerate that's expected:
`let mut notefinder = rustchord::Notefinder::new(48000)`

Then provide samples in a buffer as you process them:
`notefinder.run(&samplevec)`

After Notefinder has ran you can get the folded notes by doing:
`notefinder.get_notes()`

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
