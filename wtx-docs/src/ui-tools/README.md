# UI tools

`wtx-ui` is a standalone crate intended to allow interactions with the [`wtx`](https://github.com/c410-f3r/wtx) project through an user interface. At the current time only CLI interfaces are available.

- Embeds SQL migrations for `schema-manager`. Activation feature is called `embed-migrations`.
- Runs SQL migrations managed by `schema-manager`. Activation feature is called `schema-manager` or `schema-manager-dev`.
- Performs very basic WebSocket Client/Server operations. Activation feature is called `web-socket`.
- Makes requests to arbitrary URIs mimicking the interface of `cURL`. Activation feature is called `http-client`.