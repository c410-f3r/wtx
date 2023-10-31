set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

## fuzzingclient

cargo build --bin autobahn-server --features flate2,tokio,web-socket-handshake --release
cargo run --bin autobahn-server --features flate2,tokio,web-socket-handshake --release & cargo_pid=$!
mkdir -p .scripts/autobahn/reports/fuzzingclient
podman run \
	-p 9070:9070 \
	-v .scripts/autobahn/fuzzingclient-min.json:/fuzzingclient.json:ro \
	-v .scripts/autobahn:/autobahn \
	--name fuzzingclient \
	--net=host \
	--rm \
	docker.io/crossbario/autobahn-testsuite:0.8.2 wstest -m fuzzingclient -s fuzzingclient.json
podman rm --force --ignore fuzzingclient
kill -9 $cargo_pid

if [ $(grep -ci "failed" .scripts/autobahn/reports/fuzzingclient/index.json) -gt 0 ]
then
    exit 1
fi