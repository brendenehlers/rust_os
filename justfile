default:
    @just --list

build:
    cargo build --target ./x86_64-blog_os.json

build-release:
    cargo build --target ./x86_64-blog_os.json --release

run:
    qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin
