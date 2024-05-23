podman run \
    -d \
    --name wtx_postgres_scram \
    -e POSTGRES_DB=wtx \
    -e POSTGRES_PASSWORD=wtx \
    -p 5432:5432 \
    -v ./.test-utils/postgres.sh:/docker-entrypoint-initdb.d/setup.sh \
    docker.io/library/postgres:16

# Utils

# podman exec -it wtx_postgres_scram psql -U wtx_scram -d wtx
