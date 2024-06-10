# Config

Config files are used to configure the application at runtime.

## config file structure

```yaml
service:
    # ip:port binding
    listen: '127.0.0.1:50051'

    # enable or disable tls
    # if enabled, needs server.crt and server.key files
    tls: false

    # list of token. for demonstration purpose only !
    # auth is disabled if list is null or empty
    tokens: []
mongodb:
    # Mongodb uri with options
    uri: 'mongodb://user:password@127.0.0.1:27017/'

    # database name
    db: 'encelade'

    # collection name
    collection: 'register'
```

## profile

If a profile is exported with **REGISTER_PROFILE** variable, the file `$REGISTER_PROFILE.yaml` will be loaded if it exists.

## local.yaml

`local.yaml` will override settings loaded from a profile config.

Example:

```yaml
service:
  listen: '0.0.0.0:50051'
  tls: false
mongodb:
  uri: 'mongodb://127.0.0.1:27027/'
  db: 'encelade'
  collection: 'register'
```