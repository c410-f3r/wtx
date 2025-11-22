podman run \
    -d \
    --name wtx_mysql \
    -e MYSQL_DATABASE=wtx \
    -e MYSQL_PASSWORD=wtx \
    -e MYSQL_ROOT_HOST='%' \
    -e MYSQL_ROOT_PASSWORD=wtx \
    -e MYSQL_USER=wtx \
    -p 3306:3306 \
    -v .test-utils/my.cnf:/etc/mysql/my.cnf \
    -v .test-utils/mysql.sh:/docker-entrypoint-initdb.d/setup.sh \
    docker.io/library/mysql:9

podman run \
    --name wtx_postgres_scram \
    -d \
    -e POSTGRES_DB=wtx \
    -e POSTGRES_PASSWORD=wtx \
    -p 5432:5432 \
    -v .test-utils/postgres.sh:/docker-entrypoint-initdb.d/setup.sh \
    docker.io/library/postgres:18

# Utils

# podman exec -it wtx_postgres_scram psql -U wtx_scram -d wtx
