set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

# fuzzingclient

cargo build --example web_socket_server_echo_raw_tokio --features tokio,web-socket-handshake --release
cargo run --example web_socket_server_echo_raw_tokio --features tokio,web-socket-handshake --release & cargo_pid=$!
mkdir -p .scripts/autobahn/reports/fuzzingclient
podman run \
	-v .scripts/autobahn/fuzzingclient.json:/fuzzingclient.json:ro \
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

## fuzzingserver

cargo build --example web_socket_client_autobahn_raw_tokio --features tokio,web-socket-handshake --release
mkdir -p .scripts/autobahn/reports/fuzzingserver
podman run \
	-d \
	-p 9080:9080 \
	-v .scripts/autobahn/fuzzingserver.json:/fuzzingserver.json:ro \
	-v .scripts/autobahn:/autobahn \
	--name fuzzingserver \
	--net=host \
	docker.io/crossbario/autobahn-testsuite:0.8.2 wstest -m fuzzingserver -s fuzzingserver.json
sleep 5
cargo run --example web_socket_client_autobahn_raw_tokio --features tokio,web-socket-handshake --release -- 127.0.0.1:9080
podman rm --force --ignore fuzzingserver

if [ $(grep -ci "failed" .scripts/autobahn/reports/fuzzingserver/index.json) -gt 0 ]
then
    exit 1
fi
