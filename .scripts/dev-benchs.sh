declare -A techs
declare -A results_check
declare -A results_debug
declare -A results_opt
declare -A results_opt_size

techs["client-api-framework"]="--example client-api-framework --features client-api-framework"
techs["database-client-postgres"]="--example database-client-postgres --features database-client-postgres"
techs["grpc-client"]="--example grpc-client --features grpc-client"
techs["http-client-pool"]="--example http-client-pool --features http-client-pool"
techs["http-server-framework"]="--example http-server-framework --features http-server-framework"
techs["web-socket-client"]="--example web-socket-client --features web-socket-client"

pushd wtx-instances

for tech in "${!techs[@]}"; do
	echo "Testing '$tech'"
	results_check["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo check ${techs[$tech]}" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
	results_debug["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo build ${techs[$tech]}" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
	results_opt["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo build ${techs[$tech]} --release" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
	cargo build ${techs[$tech]} --profile deploy &> /dev/null && results_opt_size["$tech"]=$(du -h "../target/deploy/examples/$tech" | cut -f1)
done

echo -e "Technology\tCheck\tDebug\tOpt"
for tech in "${!techs[@]}"; do
	echo -e "$tech\t${results_check[$tech]}\t${results_debug[$tech]}\t${results_opt[$tech]}\t${results_opt_size[$tech]}"
done