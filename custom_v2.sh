./x.py build miri cargo-miri --stage 2
./x.py build compiler/rustc --stage 2    
mv build/x86_64-unknown-linux-gnu/stage2-tools-bin/* build/x86_64-unknown-linux-gnu/stage2/bin/
rustup toolchain link fuzz_yj build/x86_64-unknown-linux-gnu/stage2
