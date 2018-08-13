# A work-in-progress vert.x tcp eventbus bridge client in Rust

[![Build Status](https://travis-ci.org/another-s347/vertx-tcp-eventbus-bridge-client-rust.svg?branch=master)](https://travis-ci.org/another-s347/vertx-tcp-eventbus-bridge-client-rust)

## The protocol ( https://github.com/vert-x3/vertx-tcp-eventbus-bridge)

The protocol is quite simple:

* 4bytes int32 message length (big endian encoding)
* json string (encoded in UTF-8)

### Messages from the bridge -> client

Every JSON message will include a `type` key, with a string
value. Each type is shown below, along with the companion keys for
that type:

#### `type: "pong"`

`pong` requires no additional keys.

It is the response to the `ping` request from client to bridge.

####  `type: "err"`

* `message`: (string, required) The type of error, one of:
  `"access_denied"`, `"address_required"`, `"unknown_address"`,
  `"unknown_type"`

#### `type: "message"`

For a regular message, the object will also contain:

* `address`: (string, required) Destination address.
* `body`: (object, required) Message content as a JSON object.
* `headers`: (object, optional) Headers as a JSON object with String values.
* `replyAddress`: (string, optional) Address for replying to.
* `send`: (boolean, required) Will be `true` if the message is a send, `false` if a publish.

When a message from the client requests a reply, and that reply fails,
the object will instead contain:

* `failureCode`: (number, required) The failure code
* `failureType`: (string, required) The failure name
* `message`: (string, required) The message from the exception that signaled the failure

### Messages from the client -> bridge

The JSON object must contain a `type` key with a string value.  Each
type is shown below, along with the companion keys for that type:

#### `type: "send"`, `type: "publish"`

* `address`: (string, required) Destination address
* `body`: (object, required) Message content as a JSON object.
* `headers`: (object, optional) Headers as a JSON object with String values.
* `replyAddress`: (string, optional) Address for replying to.

#### `type: "register"`, `type: "unregister"`

* `address`: (string, required) Destination address
* `headers`: (object, optional) Headers as a JSON object with String values.

#### `type: "ping"`

`ping` requires no additional keys.

## Example

```rust
let mut eb=eventbus::Eventbus::connect("127.0.0.1:12345").unwrap();
eb.send("test", json!({
    "aaaa":"bbbb"
}), |res|{
    println!("callback {:?}", res);
});
```

## Dependency

### Json
* serde_json = "1.0"
* serde = "1.0"
* serde_derive = "1.0"

### Bytes
* bytes = "0.4"

### Scoped thread
* crossbeam = "0.3.2"