default:
    @just --list

install-target:
    rustup target add thumbv7em-none-eabihf

build:
    cargo build --target thumbv7em-none-eabihf

build-release:
    cargo build --target thumbv7em-none-eabihf --release
