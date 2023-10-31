set -euxo pipefail

ARG=${1:-""}
if [ "$ARG" != "ci" ]; then
	trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
fi;

cargo build --bin autobahn-client --features flate2,tokio,web-socket-handshake --release
mkdir -p .scripts/autobahn/reports/fuzzingserver
podman run \
	-d \
	-p 9080:9080 \
	-v .scripts/autobahn/fuzzingserver-min.json:/fuzzingserver.json:ro \
	-v .scripts/autobahn:/autobahn \
	--name fuzzingserver \
	--net=host \
	docker.io/crossbario/autobahn-testsuite:0.8.2 wstest -m fuzzingserver -s fuzzingserver.json
sleep 5
cargo run --bin autobahn-client --features flate2,tokio,web-socket-handshake --release
podman rm --force --ignore fuzzingserver

if [ $(grep -ci "failed" .scripts/autobahn/reports/fuzzingserver/index.json) -gt 0 ]
then
    exit 1
fi
