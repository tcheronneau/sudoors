Database creation : diesel setup
Migration creation: diesel migration generate sudos


Testing GRPC : `grpcurl -plaintext -import-path ./proto -proto sudoing.proto -d '{"hostname": "Tonic"}' 'localhost:50051' sudoing.Sudoing/Sudo`

Testing HTTP : `curl -XPOST -H "Content-Type: application/json" localhost:8000/sudo -d '{"username": "thomas", "duration": 10}'`

