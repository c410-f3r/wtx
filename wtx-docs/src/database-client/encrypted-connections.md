# Encrypted Connections

It usually isn't straightforward to stablish encrypted connections with PostgreSQL, worse yet, `wtx` has a set of limited SSL policies that doesn't allow the by-passing of invalid certificates.

The following sections will briefly demonstrate how to configure both servers and clients to establish encrypted connections using self-signed certificates with Podman or Docker. Most of the procedures can be adapted for non-containerized environments and also for certificates issued by trusted actors.

In case of doubt, always remember that a server needs a key and a certificate while both parties need a root authority certificate. Sometimes even a CA certificate isn't necessary.

## Generate certificates

Just an example, you can use other tools like `cert-manager` or other algorithms like `ed25519`.

```bash
CERTS_DIR="SOME_DIRECTORY"
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
```

## PostgreSQL

You need to place these certificates in the container at the specified location ***AND*** set the same files as read-only for the current user. As far as I can tell, there are three possible ways.

1. Create a custom Docker image.
2. List a set of volume mappings alongside some initial script.
3. Inline certificates in `docker-entrypoint-initdb.d`

Let's use option 3 for the sake of simplicity with a script named `setup.sh`.


```bash
#!/usr/bin/env bash

echo "Contents of the generated root CA certificate file" > $PGDATA/root-ca.crt
echo "Contents of the generated certificate file" > $PGDATA/cert.pem
echo "Contents of the generated key file" > $PGDATA/cert.pem

chmod 0600 $PGDATA/cert.pem $PGDATA/key.pem

cat >> "$PGDATA/postgresql.conf" <<-EOF
ssl = on
ssl_ca_file = 'root-ca.crt'
ssl_cert_file = 'cert.pem'
ssl_key_file = 'key.pem'
EOF
```

Everything should be ready on the server side.


```bash
podman run \
  --name SOME_CONTAINER_NAME \
  -d \
  -e POSTGRES_DB=SOME_DB \
  -e POSTGRES_PASSWORD=SOME_PASSWORD \
  -p 5432:5432 \
  -v SOME_DIRECTORY/setup.sh:/docker-entrypoint-initdb.d/setup.sh \
  docker.io/library/postgres:18
```

Now it is just a matter of including the root CA certificate in the `wtx` client. With everything properly configured, a successful encrypted connection should be expected.

```text
async fn tls() {
  let uri = UriRef::new("SOME_URI");
  let mut rng = ChaCha20::from_std_random().unwrap();
  let _executor = PostgresExecutor::<crate::Error, _, _>::connect_encrypted(
    &Config::from_uri(&uri).unwrap(),
    ExecutorBuffer::new(usize::MAX, &mut rng),
    &mut rng,
    TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap(),
    |stream| async {
      Ok(
        crate::misc::TokioRustlsConnector::default()
          .push_certs(include_bytes!("SOME_DIRECTORY/root-ca.crt"))
          .unwrap()
          .connect_without_client_auth(uri.hostname(), stream)
          .await
          .unwrap(),
      )
    },
  )
  .await?;
}