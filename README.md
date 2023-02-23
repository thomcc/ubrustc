# `ubrustc`: Unborrowed `rustc`

This is rustc with the borrow checker disabled. I wrote it in like, 30 minutes because [this tweet](https://twitter.com/thingskatedid/status/1628572490992877568) made me laugh.

## Example

```rs
// main.rs
fn main() {
    let s = "hi, world".to_string();
    let msg: &str = &s;
    drop(s);
    println!("Hello, memory corruption: {msg:?}");
}
```

```sh
# <install using steps in `Usage` section>
$ RUSTC=ubrustc cargo run
   Compiling ubrustc-verify v0.1.0 (/Users/thom/scratch/ubrustc-verify)
    Finished dev [unoptimized + debuginfo] target(s) in 0.12s
     Running `target/debug/ubrustc-verify`
Hello, memory corruption: "\0\0\0\0\0\0\0\0\0"
```

## Usage

You will need to be on a nightly rust (the toolchain from 2023-02-09 is known to work), and have the `rustc-dev` and `llvm-tools-preview` components installed.

Then, you can either do `./build.sh` to build the binary, which will place it somewhere inside `target/*`, or `./build.sh install`. If you're on Windows, IDK (it shouldn't be hard to install, but you likely need to set the rpath right).

Once it's installed, you use it by setting the `RUSTC` environment variable to `ubrustc` when running cargo.

Please don't use this in production. I will not be maintaining it.

## Acknowledgements

Apologies to [Will Crichton](https://twitter.com/wcrichton/status/1618096228197367808).
