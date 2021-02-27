# phone game rust

Run: `cargo run --release`

## Building for windows

`cargo build --target x86_64-pc-windows-gnu --release --bin cherry-ball`

## Building js

Install `emsdk`: https://github.com/emscripten-core/emsdk

Install and activate sdk version `1.39.20`.

Run `cargo build --target=asmjs-unknown-emscripten --release --bin cherry-ball`

The resulting `build.html` and `build.js` files will be placed in `target/asmjs-unknown-emscripten/release`. (There is probably a much nicer way of setting this up but I have not looked into it yet.)
