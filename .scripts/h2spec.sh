set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

cargo build --bin h2spec-server --features="http2,tokio"
cargo run --bin h2spec-server --features="http2,tokio" &> /tmp/h2spec-server.txt & cargo_pid=$!
sleep 1
touch /tmp/h2spec-server.xml

podman run \
	-v "/tmp/h2spec-server.xml:/tmp/h2spec-server.xml" \
	--name h2spec \
	--network host \
	--rm \
	docker.io/summerwind/h2spec:2.6.0 h2spec -j "/tmp/h2spec-server.xml" --max-header-length 800 -p 9000 -v \
		generic/1 \
		generic/2 \
		generic/3.1 \
		generic/3.2/1 \
		generic/3.2/2 \
		`#generic/3.2/3 - Priority is unsupported` \
		generic/3.3 \
		generic/3.4 \
		generic/3.5 \
		generic/3.7 \
		generic/3.8 \
		generic/3.9 \
		generic/3.10 \
		generic/4 \
		generic/5 \
		hpack \
		http2/3 \
		http2/4 \
		http2/5.1 \
		`#http2/5.3.1 - Priority is unsupported` \
		http2/5.3.2 \
		http2/5.4 \
		http2/5.5 \
		http2/6.1 \
		http2/6.2 \
		`#http2/6.3 - Priority is unsupported` \
		http2/6.4 \
		http2/6.5/1 \
		http2/6.5/2 \
		http2/6.5/3 \
		`#http2/6.5.2/1 - Server push is unsupported` \
		http2/6.5.2/2 \
		http2/6.5.2/3 \
		http2/6.5.2/4 \
		http2/6.5.2/5 \
		http2/6.5.3 \
		http2/6.7 \
		http2/6.8 \
		http2/6.9 \
		http2/6.10 \
		http2/7 \
		http2/8.1 \
		`#http2/8.2 - Server push is unsupported`

kill -9 $cargo_pid
