
set -euxo pipefail

if [ ! -d "./boringssl" ]; then
    git clone --depth 1 --branch 0.20241209.0 https://github.com/google/boringssl
fi

cargo build --bin boringssl --features tls
cd boringssl/ssl/test/runner
go test -shim-path "target/ssl/test/bssl_shim" -num-workers 1
