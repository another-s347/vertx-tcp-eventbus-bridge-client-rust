# A work-in-progress future based vert.x tcp eventbus bridge client in Rust

### Current Status

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
  `"AccessDenied"`, `"AddressRequired"`, `"UnknownAddress"`,
  `"UnknownType"`

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
        let task = future::Eventbus::connect(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
        let task = task.and_then(|(eventbus, readstream, writestream)| {
            tokio::spawn(readstream.into_future().map(|_|()).map_err(|e|()));
            tokio::spawn(writestream.into_future().map(|_|()).map_err(|e|()));
            futures::future::ok(eventbus)
        });
        let task = task.and_then(|eventbus:Eventbus|{
            let test_reply=eventbus.send_reply("test".to_string(),json!({
                "aaaa":"bbbb"
            })).unwrap().and_then(|response|{
                println!("{:?}",response);
                futures::future::ok(())
            });
            test_reply
        });
        tokio::run(task.map_err(|e|()));
```

## Dependency

### future
* futures = "0.1"

### Json
* serde_json = "1.0"
* serde = "1.0"
* serde_derive = "1.0"

### Bytes
* bytes = "0.4"