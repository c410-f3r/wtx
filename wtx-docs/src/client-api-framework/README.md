# Client API Framework

A flexible client API framework for writing asynchronous, fast, organizable, scalable and maintainable applications. Supports several data formats, transports and custom parameters.

Checkout the `wtx-apis` project to see a collection of APIs based on `wtx`.

To use this functionality, it is necessary to activate the `client-api-framework` feature.

## Objective

It is possible to directly decode responses using built-in methods provided by some transport implementations like `reqwest` or `surf` but as complexity grows, the cost of maintaining large sets of endpoints with ad-hoc solutions usually becomes unsustainable. Based on this scenario, `wtx` comes into play to organize and centralize data flow in a well-defined manner to increase productivity and maintainability.

For API consumers, the calling convention of `wtx` endpoints is based on fluent interfaces which makes the usage more pleasant and intuitive.

Moreover, the project may in the future create automatic bindings for other languages in order to avoid having duplicated API repositories.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/client-api-framework.rs}}
```