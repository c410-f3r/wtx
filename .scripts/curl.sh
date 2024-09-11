set -euxo pipefail

PWD="$(pwd)"

if [ -d "./curl" ]; then
    cd curl/tests
else
    #sudo apt install autoconf libnghttp2-dev libtool libpsl-dev -y
    git clone --branch curl-8_9_1 --depth 1 https://github.com/curl/curl
    cd curl
    autoreconf -fi
    ./configure --enable-debug --with-nghttp2 --without-ssl
    make
    cd tests
fi

cargo build --bin wtx-ui --features http-client --release
./runtests.pl -d -n -c "$PWD/../../target/release/wtx-ui"