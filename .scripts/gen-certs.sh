# https://stackoverflow.com/questions/76049656/unexpected-notvalidforname-with-rusts-tonic-with-tls

set -euxo pipefail

CERTS_DIR="$(dirname $0)/../.certs"
TEST_UTILS_DIR="$(dirname $0)/../.test-utils"

db_file_init() {
    local local_file=$1;
    local remote_data_dir=$2

    echo "#!/usr/bin/env bash" > $local_file
    echo "set -euxo pipefail" >> $local_file
    echo "DATA_DIR=\"$remote_data_dir\"" >> $local_file
    echo "echo \"$(cat $CERTS_DIR/root-ca.crt)\" > \$DATA_DIR/root-ca.crt" >> $local_file
    echo "echo \"$(cat $CERTS_DIR/cert.pem)\" > \$DATA_DIR/cert.pem" >> $local_file
    echo "echo \"$(cat $CERTS_DIR/key.pem)\" > \$DATA_DIR/key.pem" >> $local_file
}

openssl req -newkey rsa:2048 -nodes -subj "/C=FI/CN=vahid" -keyout $CERTS_DIR/key.pem -out $CERTS_DIR/key.csr
openssl x509 -signkey $CERTS_DIR/key.pem -in $CERTS_DIR/key.csr -req -days 1825 -out $CERTS_DIR/cert.pem
openssl req -x509 -sha256 -nodes -subj "/C=FI/CN=vahid" -days 1825 -newkey rsa:2048 -keyout $CERTS_DIR/root-ca.key -out $CERTS_DIR/root-ca.crt
cat <<'EOF' >> $CERTS_DIR/localhost.ext
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
subjectAltName = @alt_names
[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
EOF
openssl x509 -req -CA $CERTS_DIR/root-ca.crt -CAkey $CERTS_DIR/root-ca.key -in $CERTS_DIR/key.csr -out $CERTS_DIR/cert.pem -days 1825 -CAcreateserial -extfile $CERTS_DIR/localhost.ext
rm $CERTS_DIR/key.csr
rm $CERTS_DIR/localhost.ext
rm $CERTS_DIR/root-ca.srl

# MySQL

MYSQL_LOCAL_FILE="$TEST_UTILS_DIR/mysql.sh"
MYSQL_REMOTE_DATA_DIR="/var/lib/mysql"

db_file_init $MYSQL_LOCAL_FILE $MYSQL_REMOTE_DATA_DIR

echo "chown mysql:mysql $MYSQL_REMOTE_DATA_DIR/cert.pem $MYSQL_REMOTE_DATA_DIR/key.pem" >> $MYSQL_LOCAL_FILE
echo "chmod 0600 $MYSQL_REMOTE_DATA_DIR/cert.pem $MYSQL_REMOTE_DATA_DIR/key.pem" >> $MYSQL_LOCAL_FILE

echo "mysql -uroot -p\"\$MYSQL_ROOT_PASSWORD\" -e \"CREATE USER 'no_password'@'%'; GRANT SELECT ON wtx.* TO 'no_password'@'%';\"" >> $MYSQL_LOCAL_FILE

# PostgreSQL

POSTGRES_LOCAL_FILE="$TEST_UTILS_DIR/postgres.sh"
POSTGRES_REMOTE_DATA_DIR="\$PGDATA"

db_file_init $POSTGRES_LOCAL_FILE $POSTGRES_REMOTE_DATA_DIR

echo "chmod 0600 \$PGDATA/key.pem" >> $POSTGRES_LOCAL_FILE

echo "cat >> \"$POSTGRES_REMOTE_DATA_DIR/postgresql.conf\" <<-EOF
ssl = on
ssl_ca_file = 'root-ca.crt'
ssl_cert_file = 'cert.pem'
ssl_key_file = 'key.pem'
EOF" >> $POSTGRES_LOCAL_FILE

echo "cat > \"$POSTGRES_REMOTE_DATA_DIR/pg_hba.conf\" <<-EOF
host    all wtx_scram   0.0.0.0/0   scram-sha-256
host    all wtx_scram       ::0/0   scram-sha-256
EOF

psql -v ON_ERROR_STOP=1 --username "\$POSTGRES_USER" <<-EOF
    SET password_encryption TO 'scram-sha-256';
    CREATE ROLE wtx_scram PASSWORD 'wtx' LOGIN;
    GRANT ALL ON DATABASE wtx TO wtx_scram;
    ALTER DATABASE wtx OWNER TO wtx_scram;
EOF" >> $POSTGRES_LOCAL_FILE