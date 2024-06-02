# Encelade Suite - Register backend

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

## Usage

### grpcurl

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