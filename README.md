# Encelade Suite - Register backend

![encelade illustration](docs/encelade.png) Register backend implementing [register.proto](proto/register.proto).

## Development

### Setup

Encelade Suite - Register backend is using rust, protobuf and MongoDB.

### Compilation / packaging

```bash
docker build -t encelade-suite-register-backend:latest .
```

### Run

#### MongoDB

MongoDB is used for storage. You can use a free instance from [MongoDB Atlas](https://www.mongodb.com/cloud/atlas/register) or use a local mongo instance with replicaset support.

```bash
docker run -it --rm --name mongo -p 27017:27017 -e MONGODB_REPLICA_SET_MODE=primary -e MONGODB_REPLICA_SET_NAME=rs0 -e MONGODB_INITIAL_PRIMARY_HOST=127.0.0.1 -e ALLOW_EMPTY_PASSWORD=yes bitnami/mongodb
```

#### Register backend

```bash
# create config file
cat <<EOF > config/demo.yaml
service:
  listen: '0.0.0.0:50051'
  tls: false
mongodb:
  uri: 'mongodb://172.17.0.1:27017/?directConnection=true&connectTimeoutMS=2000&serverSelectionTimeoutMS=2000'
  db: 'encelade'
  collection: 'register'
EOF

docker run -it --rm -v $(pwd)/config/demo.yaml:/config/local.yaml -p 50051:50051 encelade-suite-register-backend:latest
```

#### TLS

If tls is enabled, `config/server.key` and `config/server.crt` files will be required.

##### Create your own certificates

```bash
# eg with your own root ca

cd config
cat << EOF > server.v3.ext
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
IP.2 = 10.0.2.2
IP.3 = 172.17.0.1
EOF

# generate root CA
openssl genrsa -out ca.key 4096
openssl req -x509 -new -nodes -key ca.key -days 1826 -out ca.crt -subj '/CN=Encelade Suite Root CA/C=CA/ST=Quebec/L=Montreal/O=Pygoscelis'

# trust your root CA
sudo trust anchor --store ca.crt

# generate server.key and server.crt signed by your root CA
openssl req -new -nodes -out server.csr -newkey rsa:4096 -keyout server.key -subj '/CN=Encelade Register Backend/C=CA/ST=Quebec/L=Montreal/O=Pygoscelis'
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 730 -sha256 -extfile server.v3.ext

# to untrust your root CA
# sudo trust anchor --remove ca.crt
```

##### Configure tls

Update `service.tls` key from your config file:

```yaml
service:
  tls: true
```

Once your service is started, you should see a log line:

```bash
use tls: true
```

##### Illegal SNI

This project is using rustls which will not be able to do the handshake if the SNI hostname is an IP. This is not an issue with rustls but with the client itself. If the client is not able to handle the SNI properly, please use an hostname/fqdn instead of an IP and update **alt_names** in `server.v3.ext` file.

Log example:

```bash
2024-06-10T00:15:46.363931Z  WARN rustls::msgs::handshake: Illegal SNI hostname received "127.0.0.1"
```

## Usage

### grpcurl

*NOTE: remove `-plaintext` argument in below commands if tls is enabled.*

```bash
# Information about server grpc proto
grpcurl -proto ./proto/register.proto -plaintext 127.0.0.1:50051 list
grpcurl -proto ./proto/register.proto -plaintext 127.0.0.1:50051 describe register.Register

# Create a new draft
ID=$(grpcurl -proto ./proto/register.proto -d '{"summary": "test!"}' -plaintext 127.0.0.1:50051 register.Register/NewDraft | jq -r .id | sed 's/ObjectId("\(.*\)")/\1/')

# Delete a draft
# grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'"}' -plaintext 127.0.0.1:50051 register.Register/DeleteDraft

# Update a draft
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "summary": "update my test!"}' -plaintext 127.0.0.1:50051 register.Register/UpdateDraft

# Submit a draft; it will not be possible to remove it after this call
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'"}' -plaintext 127.0.0.1:50051 register.Register/SubmitDraft

# Client is inside office
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "time": "2024-04-23T21:00:00Z" }' -plaintext 127.0.0.1:50051 register.Register/CollectClientInside

# Client collect products and sign the register
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "signer": {"name": "client", "signature": "cs"} }' -plaintext 127.0.0.1:50051 register.Register/CollectClientSignature

# Client is outside office
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "time": "2024-04-23T21:01:00Z" }' -plaintext 127.0.0.1:50051 register.Register/CollectClientOutside

# PQRS sign the register. Collect is done.
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "signer": {"name": "pqrs", "signature": "ps"} }' -plaintext 127.0.0.1:50051 register.Register/CollectPqrsSignature

# Client is inside office
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "time": "2024-04-23T23:00:00Z" }' -plaintext 127.0.0.1:50051 register.Register/ReturnClientInside

# Client return products and sign the register
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "signer": {"name": "client", "signature": "cs"} }' -plaintext 127.0.0.1:50051 register.Register/ReturnClientSignature

# Client is outside office
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "time": "2024-04-23T23:05:00Z" }' -plaintext 127.0.0.1:50051 register.Register/ReturnClientOutside

# PQRS sign the register. Return is done.
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'", "signer": {"name": "pqrs", "signature": "ps"} }' -plaintext 127.0.0.1:50051 register.Register/ReturnPqrsSignature

# PQRS checkout is completed
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'" }' -plaintext 127.0.0.1:50051 register.Register/Complete

# Search a specific record in the register
grpcurl -proto ./proto/register.proto -d '{"id": "'$ID'"}' -plaintext 127.0.0.1:50051 register.Register/SearchById

# Search all records with a specific state
#  (eg: COMPLETED state)
grpcurl -proto ./proto/register.proto -d '{"states": ["COMPLETED"]}' -plaintext 127.0.0.1:50051 register.Register/Search
#  (eg: COMPLETED state and created between 2 dates)
grpcurl -proto ./proto/register.proto -d '{"states": ["COMPLETED"], "range": { "begin":"1970-01-01T00:00:00Z", "end":"1970-01-02T00:00:00Z" }}' -plaintext 127.0.0.1:50051 register.Register/Search

# Watch all events in the register
grpcurl -proto ./proto/register.proto -d '{}' -plaintext 127.0.0.1:50051 register.Register/Watch
```