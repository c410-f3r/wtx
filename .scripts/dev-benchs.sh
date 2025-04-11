declare -A techs
declare -A results_check
declare -A results_debug
declare -A results_opt

techs["Client API Framework"]="--example client-api-framework --features client-api-framework"
techs["gRPC Client"]="--example grpc-client --features grpc-client"
techs["HTTP Client Pool"]="--example http-client-pool --features http-client-pool"
techs["HTTP Server Framework"]="--example http-server-framework --features http-server-framework"
techs["Postgres Client"]="--example database-client-postgres --features database-client-postgres"
techs["WebSocket Client"]="--example web-socket-client --features web-socket-client"

pushd wtx-instances

for tech in "${!techs[@]}"; do
	results_check["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo check ${techs[$tech]}" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
	results_debug["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo build ${techs[$tech]}" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
	results_opt["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo build ${techs[$tech]} --release" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
done

echo -e "Technology\tCheck\tDebug\tOpt"
for tech in "${!techs[@]}"; do
	echo -e "$tech\t${results_check[$tech]}\t${results_debug[$tech]}\t${results_opt[$tech]}"
done