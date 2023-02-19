#!/bin/sh
rm -rf target
exec /home/ekleog/.rustup/toolchains/stage1/bin/rustc \
    src/lib.rs \
    --emit=link \
    -C opt-level=2 \
    --out-dir ./target -Ctarget-cpu=native -Cdebuginfo=2 -Coverflow_checks -Clink-dead-code -Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-level=3 -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares -Zsanitizer=address
