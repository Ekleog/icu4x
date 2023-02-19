#!/bin/sh
#~/prog/rust/build/x86_64-unknown-linux-gnu/llvm/bin/llc main.bc
export PATH="$HOME/prog/rust/build/x86_64-unknown-linux-gnu/llvm/bin:$PATH"
llvm-dis main.bc --preserve-ll-uselistorder -o disas.ll
llvm-as disas.ll --preserve-bc-uselistorder -o disas.bc
llc disas.bc