set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

cargo build --example http2-server-tokio --features="async-send,http2,parking_lot,pool,tokio" --release
cargo run --example http2-server-tokio --features="async-send,http2,parking_lot,pool,tokio" --release & cargo_pid=$!
sleep 1
#podman run \
#	-p 9000:9000 \
#	--name h2spec \
#	--network host \
#	--rm \
#	docker.io/summerwind/h2spec:2.6.0 h2spec -p 9000 http2
kill -9 $cargo_pid
