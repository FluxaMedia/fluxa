#!/bin/sh
set -e
cd "$(dirname "$0")"
openssl req -x509 -newkey rsa:2048 -nodes -keyout ca.key -out ca.pem -days 36500 -subj "/CN=Fluxa Mock CA" -addext "basicConstraints=critical,CA:TRUE"
openssl req -newkey rsa:2048 -nodes -keyout server.key -out server.csr -config san.cnf
openssl x509 -req -in server.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out server.pem -days 36500 -extfile san.cnf -extensions ext
openssl pkcs8 -topk8 -nocrypt -in server.key -out server.pk8.pem
rm -f server.csr ca.srl server.key ca.key
