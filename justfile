run:
    mold -run cargo run --features fast_compile

dev:
    mold -run cargo run --features dev fast_compile

build:
    mold -run cargo run --release --features release

