# mogwai-realworld

Hello, this is the [realworld](https://github.com/gothinkster/realworld) frontend 
application demo for the [mogwai](https://github.com/schell/mogwai) rust library. 

This is a work in progress. Feel free to contribute or criticize - all feedback is 
welcome :)

## getting started 
First you'll need [`rustup`](https://rustup.rs/), which manages versions of `rustc` 
and friends. 

Then you'll need [`wasm-pack`](https://rustwasm.github.io/docs/wasm-pack/) which 
uses `rustc` to cross-compile rust code to WASM _.

Then you'll need a simple file server. I like to use `basic-http-server`, which can 
be installed with `cargo`:
```bash
cargo install basic-http-server
```

## building 
First build:

```bash
wasm-pack build --target web
```

then serve:

```bash
basic-http-server -a 127.0.0.1:8888
```

and then visit http://127.0.0.1:8888 in your browser. 

## fin 

Happy Hacking! 🚧☕☕🚧
