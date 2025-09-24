# HTTP Server Framework

A small and fast to compile framework that can interact with many built-in features.

* Databases
* JSON
* Middlewares
* Streaming
* URI router
* WebSocket

If dynamic or nested routes are needed, then please activate the `matchit` feature. Without it, only simple and flat routes will work.

To use this functionality, it is necessary to activate the `http-server-framework` feature.

![HTTP/2 Benchmarks](https://i.imgur.com/lUOX3iM.png)

## Endpoints

Under the hood every endpoint transforms a `Request` (Body, Method, Headers, Uri) into a `Response` (Body, Headers, StatusCode) and users can perform a finer control over this process.

### Input

You will get what you declare as input. Everything else is previously sanitized.

```rust,edition2024
async fn health() {}
```

The above example accepts nothing `()` as input parameter, which automatically implies a sanitization of the received request. 

```rust,edition2024
extern crate wtx;

use wtx::http::{ReqResBuffer, server_framework::State};

async fn print_request(state: State<'_, (), (), ReqResBuffer>) {
  println!("Request: {:?}", &state.req);
}
```

On the other hand, the above example gives you access to the full request. This also implies that you should be responsable for data management.

### Output

Determines how responses are constructed. Similar to input handling, the output defines what clients receive.

For instance, this endpoint returns a simple `Hello` as the response body along side an implicit 200 OK status. No headers are sent.

```rust,edition2024
async fn hello() -> &'static str {
  "Hello"
}
```

There are many other types of outputs that perform different operations. Please see the documentation for a full listening.

### Buffers

A key consideration is that the buffers used for receiving requests are the same ones utilized for constructing responses, which means, among other things:

1. If the response body is equal to or smaller than the request body in size, a memory allocation can be avoided, potentially improving runtime performance.
2. If you use `State` with `VerbatimParams` or `DynParams::Verbatim`, then you should probably be careful to avoid leaking request information into responses.

```rust,edition2024
extern crate wtx;

use wtx::http::{ReqResBuffer, server_framework::{State, VerbatimParams}};

// The response will contain the same headers and data received from the request. Basically an echo.
async fn echo(_: State<'_, (), (), ReqResBuffer>) -> wtx::Result<VerbatimParams> {
  Ok(VerbatimParams::default())
}
```

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/http-server-framework-examples/http-server-framework.rs}}
```