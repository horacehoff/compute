RUSTFLAGS="-Cprofile-use=$(readlink -f ./merged.profdata) -Cllvm-args=-pgo-warn-missing-function" cargo build --release