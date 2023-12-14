podman run \
    -d \
    --name wtx_postgres_md5 \
    -e POSTGRES_DB=wtx \
    -e POSTGRES_HOST_AUTH_METHOD=md5 \
    -e POSTGRES_INITDB_ARGS="--auth-host=md5" \
    -e POSTGRES_PASSWORD=wtx \
    -p 5432:5432 \
    -v .test-utils/postgres.sh:/docker-entrypoint-initdb.d/setup.sh \
    docker.io/library/postgres:16

podman run \
    -d \
    --name wtx_postgres_scram \
    -e POSTGRES_DB=wtx \
    -e POSTGRES_HOST_AUTH_METHOD=scram-sha-256 \
    -e POSTGRES_INITDB_ARGS="--auth-host=scram-sha-256" \
    -e POSTGRES_PASSWORD=wtx \
    -p 5433:5432 \
    -v .test-utils/postgres.sh:/docker-entrypoint-initdb.d/setup.sh \
    docker.io/library/postgres:16

# Utils

# podman exec -it wtx_postgres_md5 psql -U wtx_md5 -d wtx
# podman exec -it wtx_postgres_scram psql -U wtx_scram -d wtx
