#!/bin/sh
rm -rf target
exec /home/ekleog/.rustup/toolchains/stage1/bin/rustc src/lib.rs --emit=link -C opt-level=2 --out-dir ./target -L dependency=/home/ekleog/prog/2023-02-16-17-24-30-010/target/fuzz/build_fdfb72ebb347a346/release/deps --cfg fuzzing -Cdebug-assertions -Ctarget-cpu=native -Cdebuginfo=2 -Coverflow_checks -Clink-dead-code -Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-level=3 -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares -Zsanitizer=address
