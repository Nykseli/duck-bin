#!/bin/sh

# cargo install cargo-watch

cargo watch \
    --features hot_reload \
    -w resources \
    -w src \
    -w templates \
    -x run
