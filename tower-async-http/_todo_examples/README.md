# axum-http-server

A simple hello world example for HTTP services,
which showcases how you can have your custom middleware and
simple web service.

## Running the example

```
cargo run --example axum-http-server
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

# hyper-http-server

This example contains an example on how to use a custom tower-async service
as your http service in a stacked tower-async app.

## Endpoints

- `GET /fast` — Simulate a fast endpoint
- `GET /slow` — Simulate a slow endpoint

If you first run the `/slow` endpoint and then immediately the `/fast` one in another shell,
you should be getting a `429` response due to the rate limit reached.

## Running the example

```
RUST_LOG=hyper_http_server=trace,tower_async_http=trace \
    cargo run --example hyper-http-server
```
