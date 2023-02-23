#!/usr/bin/env bash
set -e

abs_path() {
    local path="$1"
    (unset CDPATH && cd "$path" > /dev/null && pwd)
}

cd "$(abs_path "$(dirname "$0")")"


host=$(rustc --version --verbose | grep "^host:" | cut -d ' ' -f 2)
sysroot=$(rustc --print sysroot)
libdir="$sysroot/lib/rustlib/$host/lib"

RUSTFLAGS="-Zunstable-options -Zbinary-dep-depinfo -Clink-args=-Wl,-rpath,$libdir $RUSTFLAGS"

if [[ "$1" = "install" ]]; then
    env RUSTFLAGS="$RUSTFLAGS" cargo install --path . --target "$host"
else
    env RUSTFLAGS="$RUSTFLAGS" cargo build --release -p ubrustc --bin ubrustc --target "$host"
fi
