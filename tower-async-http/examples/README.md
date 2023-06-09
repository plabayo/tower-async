# axum-http-server

A simple hello world example for HTTP services,
which showcases how you can have your custom middleware and
simple web service.

## Running the example

```
cargo run --example axum-key-value-store
```

# axum-key-value-store

This examples contains a simple key/value store with an HTTP API built using axum.

## Endpoints

- `GET /:key` - Look up a key. If the key doesn't exist it returns `404 Not Found`
- `POST /:key` - Insert a key. The value is the request body.

## Running the example

```
RUST_LOG=axum_key_value_store=trace,tower_async_http=trace \
    cargo run --example axum-key-value-store
```
