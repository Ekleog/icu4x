#!/bin/sh
rm -rf target
# works: type=llvm-ir
# works: type=asm
# works: type=obj
# does not work: type=link
exec /home/ekleog/.rustup/toolchains/stage1/bin/rustc \
    src/main.rs \
    --emit=link \
    -C opt-level=1 \
    --out-dir ./target \
    -C debuginfo=1 \
    -C codegen-units=2 \
    -Clink-dead-code \
    -Cpasses=sancov-module \
    -Cllvm-args=-sanitizer-coverage-inline-8bit-counters \
    -Cllvm-args=-sanitizer-coverage-level=3 \
    -Cllvm-args=-sanitizer-coverage-pc-table \
    -Zsanitizer=address \
    -Z no-parallel-llvm \
    -C save-temps