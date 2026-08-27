#!/usr/bin/env bash
# Cargo target runner for the kernel.
#
# `bootimage runner` picks `run-args` or `test-args` by checking whether the
# executable lives in a directory named `deps`. Nightly cargo writes test
# executables to `build/<pkg>/<hash>/out/` instead, so tests were launched with
# the run configuration: no isa-debug-exit device and no `-display none`, which
# left a QEMU window open that `exit_qemu` could never close.
#
# Test executables still carry a 16-hex-digit hash suffix that the uplifted
# `cargo run` binary does not, so detect them by name and hand bootimage a path
# below a `deps` directory.
set -eu

exe=$1
shift

name=$(basename "$exe")
if [[ $name =~ -[0-9a-f]{16}$ ]]; then
    deps=$(dirname "$exe")/deps
    mkdir -p "$deps"
    ln -f "$exe" "$deps/$name"
    exe=$deps/$name
fi

exec bootimage runner "$exe" "$@"
