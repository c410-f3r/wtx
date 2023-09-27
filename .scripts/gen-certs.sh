# https://stackoverflow.com/questions/76049656/unexpected-notvalidforname-with-rusts-tonic-with-tls

openssl req -newkey rsa:2048 -nodes -subj "/C=FI/CN=vahid" -keyout key.pem -out key.csr
openssl x509 -signkey key.pem -in key.csr -req -days 365 -out cert.pem
openssl req -x509 -sha256 -nodes -subj "/C=FI/CN=vahid" -days 1825 -newkey rsa:2048 -keyout root-ca.key -out root-ca.crt
cat <<'EOF' >> localhost.ext
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
subjectAltName = @alt_names
[alt_names]
DNS.1 = server
IP.1 = 127.0.0.1
EOF
openssl x509 -req -CA root-ca.crt -CAkey root-ca.key -in key.csr -out cert.pem -days 365 -CAcreateserial -extfile localhost.ext
rm localhost.ext
rm key.csr