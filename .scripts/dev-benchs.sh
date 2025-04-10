declare -A techs
declare -A results_check
declare -A results_debug
declare -A results_opt

techs["Client API Framework"]="--example client-api-framework --features 'serde wtx/client-api-framework wtx/http-client-pool wtx/serde_json wtx/web-socket-handshake wtx-macros'"
techs["Database Client"]="--example database-client-postgres --features 'postgres'"
techs_order=("Client API Framework" "Database Client")

pushd wtx-instances

for tech in "${techs_order[@]}"; do
	results_check["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo check ${techs[$tech]}" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
	results_debug["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo build ${techs[$tech]}" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
	results_opt["$tech"]=$(hyperfine --prepare "cargo clean" --runs 1 --warmup 1 "cargo build ${techs[$tech]} --release" --export-json /tmp/bench.json &> /dev/null && cat /tmp/bench.json | jq -r ".results[0].mean")
done

echo -e "Technology\tCheck\tDebug\tOpt"
for tech in "${techs_order[@]}"; do
	echo -e "$tech\t${results_check[$tech]}\t${results_debug[$tech]}\t${results_opt[$tech]}"
done