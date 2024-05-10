# Backend for Register

## Usage

### grpcurl

```bash
# Information about server grpc proto
grpcurl -proto ../proto/register.proto -plaintext 127.0.0.1:50051 list
grpcurl -proto ../proto/register.proto -plaintext 127.0.0.1:50051 describe register.Register

# Create a new draft
ID=$(grpcurl -proto ../proto/register.proto -d '{"summary": "test!"}' -plaintext 127.0.0.1:50051 register.Register/NewDraft | jq -r .id | sed 's/ObjectId("\(.*\)")/\1/')

# Delete a draft
# grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'"}' -plaintext 127.0.0.1:50051 register.Register/DeleteDraft

# Update a draft
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "summary": "update my test!"}' -plaintext 127.0.0.1:50051 register.Register/UpdateDraft

# Submit a draft; it will not be possible to remove it after this call
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'"}' -plaintext 127.0.0.1:50051 register.Register/SubmitDraft

# Client is inside office
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "time": "2024-04-23T21:00:00Z" }' -plaintext 127.0.0.1:50051 register.Register/CollectClientInside

# Client collect products and sign the register
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "signer": {"name": "client", "signature": "cs"} }' -plaintext 127.0.0.1:50051 register.Register/CollectClientSignature

# Client is outside office
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "time": "2024-04-23T21:01:00Z" }' -plaintext 127.0.0.1:50051 register.Register/CollectClientOutside

# PQRS sign the register. Collect is done.
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "signer": {"name": "pqrs", "signature": "ps"} }' -plaintext 127.0.0.1:50051 register.Register/CollectPqrsSignature

# Client is inside office
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "time": "2024-04-23T23:00:00Z" }' -plaintext 127.0.0.1:50051 register.Register/ReturnClientInside

# Client return products and sign the register
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "signer": {"name": "client", "signature": "cs"} }' -plaintext 127.0.0.1:50051 register.Register/ReturnClientSignature

# Client is outside office
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "time": "2024-04-23T23:05:00Z" }' -plaintext 127.0.0.1:50051 register.Register/ReturnClientOutside

# PQRS sign the register. Return is done.
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'", "signer": {"name": "pqrs", "signature": "ps"} }' -plaintext 127.0.0.1:50051 register.Register/ReturnPqrsSignature

# PQRS checkout is completed
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'" }' -plaintext 127.0.0.1:50051 register.Register/Complete

# Search a specific record in the register
grpcurl -proto ../proto/register.proto -d '{"id": "'$ID'"}' -plaintext 127.0.0.1:50051 register.Register/SearchById

# Search all records with a specific state (eg: COMPLETED state)
grpcurl -proto ../proto/register.proto -d '{"states": ["COMPLETED"]}' -plaintext 127.0.0.1:50051 register.Register/Search

# Watch all events in the register
grpcurl -proto ../proto/register.proto -d '{}' -plaintext 127.0.0.1:50051 register.Register/Watch
```