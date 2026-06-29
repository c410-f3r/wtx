#!/usr/bin/env bash

podman container rm -fi wtx_postgres_scram
podman run \
    --name wtx_postgres_scram \
    -d \
    -e POSTGRES_DB=wtx \
    -e POSTGRES_PASSWORD=wtx \
    --network host \
    -v .test-utils/postgres.sh:/docker-entrypoint-initdb.d/setup.sh:Z \
    docker.io/library/postgres:18

# podman exec -it wtx_postgres_scram psql -U wtx_scram -d wtx