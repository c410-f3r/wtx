# https://stackoverflow.com/questions/76049656/unexpected-notvalidforname-with-rusts-tonic-with-tls

set -euxo pipefail

CERTS_DIR="$(dirname $0)/../.certs"
TEST_UTILS_DIR="$(dirname $0)/../.test-utils"

openssl req -newkey rsa:2048 -nodes -subj "/C=FI/CN=vahid" -keyout $CERTS_DIR/key.pem -out $CERTS_DIR/key.csr
openssl x509 -signkey $CERTS_DIR/key.pem -in $CERTS_DIR/key.csr -req -days 365 -out $CERTS_DIR/cert.pem
openssl req -x509 -sha256 -nodes -subj "/C=FI/CN=vahid" -days 365 -newkey rsa:2048 -keyout $CERTS_DIR/root-ca.key -out $CERTS_DIR/root-ca.crt
cat <<'EOF' >> $CERTS_DIR/localhost.ext
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
subjectAltName = @alt_names
[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
EOF
openssl x509 -req -CA $CERTS_DIR/root-ca.crt -CAkey $CERTS_DIR/root-ca.key -in $CERTS_DIR/key.csr -out $CERTS_DIR/cert.pem -days 365 -CAcreateserial -extfile $CERTS_DIR/localhost.ext
rm $CERTS_DIR/localhost.ext
rm $CERTS_DIR/key.csr

# PostgreSQL

POSTGRES_FILE="$TEST_UTILS_DIR/postgres.sh"

echo "#!/usr/bin/env bash" > $POSTGRES_FILE

echo "set -euxo pipefail" >> $POSTGRES_FILE

echo "echo \"$(cat $CERTS_DIR/root-ca.crt)\" > \$PGDATA/root-ca.crt" >> $POSTGRES_FILE
echo "echo \"$(cat $CERTS_DIR/cert.pem)\" > \$PGDATA/cert.pem" >> $POSTGRES_FILE
echo "echo \"$(cat $CERTS_DIR/key.pem)\" > \$PGDATA/key.pem" >> $POSTGRES_FILE

echo "chmod 0600 \$PGDATA/key.pem" >> $POSTGRES_FILE

echo "cat >> \"\$PGDATA/postgresql.conf\" <<-EOF
ssl = on
ssl_ca_file = 'root-ca.crt'
ssl_cert_file = 'cert.pem'
ssl_key_file = 'key.pem'
EOF" >> $POSTGRES_FILE

echo "cat > \"\$PGDATA/pg_hba.conf\" <<-EOF
# TYPE  DATABASE        USER            ADDRESS                 METHOD
host    all             wtx_md5        0.0.0.0/0            md5
host    all             wtx_scram      0.0.0.0/0            scram-sha-256
host    all             wtx_md5        ::0/0                md5
host    all             wtx_scram      ::0/0                scram-sha-256

local    all             all                                md5
EOF

psql -v ON_ERROR_STOP=1 --username "\$POSTGRES_USER" <<-EOF
    CREATE ROLE wtx_md5 PASSWORD 'wtx' LOGIN;
    SET password_encryption TO 'scram-sha-256';
    CREATE ROLE wtx_scram PASSWORD 'wtx' LOGIN;
EOF" >> $POSTGRES_FILE