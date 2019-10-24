# 🗂️ `kv`

## Overview

The `kv` subcommand allows you to store application data in the Cloudflare network to be accessed from Workers, using
[Workers KV](https://www.cloudflare.com/products/workers-kv/).
KV operations are scoped to your account, so in order to use any of these commands, you need to:

* have a Wrangler project set up with your `account_id` configured in the `wrangler.toml`
* call commands from within a Wrangler project directory.

## Getting Started

To use Workers KV with your Worker, the first thing you must do is create a KV namespace. This is done with
the `kv:namespace` subcommand.

The `kv:namespace` subcommand takes as a new binding name as an argument. It will create a Worker KV namespace
whose title is a concatenation of your Worker's name (from `wrangler.toml`) and the binding name you provide:

```console
$ wrangler kv:namespace create "MY_KV"
🌀  Creating namespace with title "worker-MY_KV"
✨  Success: WorkersKvNamespace {
    id: "e29b263ab50e42ce9b637fa8370175e8",
    title: "worker-MY_KV",
}
✨  Add the following to your wrangler.toml:
kv-namespaces = [
         { binding = "MY_KV", id = "e29b263ab50e42ce9b637fa8370175e8" }
]
```

Make sure to add the `kv-namespaces` output above to your `wrangler.toml`. You can now
access it from a Worker with code like:

```js
let value = await MY_KV.get("my-key");
```

The full KV API for Workers can be found [here](https://developers.cloudflare.com/workers/reference/storage/).

To put a value to your KV namespace via Wrangler, use the `kv:key put` subcommand.

```console
$ wrangler kv:key put --binding=MY_KV "key" "value"
✨  Success
```

You can also specify which namespace to put your key-value pair into using `--namespace-id` instead of `--binding`:

```console
$ wrangler kv:key put --namespace-id=e29b263ab50e42ce9b637fa8370175e8 "key" "value"
✨  Success
```

Additionally, KV namespaces can be used with [environments](./environments.md)! This is useful for when you have code that refers to
a KV binding like `MY_KV`, and you want to be able to have these bindings point to different namespaces (like
one for staging and one for production). So, if you have a `wrangler.toml` with two environments:

```toml
[env.staging]
kv-namespaces = [
         { binding = "MY_KV", id = "e29b263ab50e42ce9b637fa8370175e8" }
]

[env.production]
kv-namespaces = [
         { binding = "MY_KV", id = "a825455ce00f4f7282403da85269f8ea" }
]
```

To insert a value into a specific KV namespace, you can use
```console
$ wrangler kv:key put --env=staging --binding=MY_MV "key" "value"
✨  Success
```

Since `--namespace-id` is always unique (unlike binding names), you don't need to pass environment variables for them (they will be unused).

There are way more helpful Wrangler subcommands for interacting with Workers KV, like ones for bulk uploads and deletes--check them out below!

## Concepts

Most `kv` commands require you to specify a namespace. A namespace can be specified in two ways:

1. With a `--binding`:
    ```sh
    wrangler kv:key get --binding=MY_KV "my key"
    ```
1. With a `--namespace_id`:
    ```sh
    wrangler kv:key get --namespace-id=06779da6940b431db6e566b4846d64db "my key"
    ```

Most `kv` subcommands also allow you to specify an environment with the optional `--env` flag. This allows you to publish workers running the same code but with different namespaces. For example, you could use separate staging and production namespaces for KV data in your `wrangler.toml`:

```toml
type = "webpack"
name = "my-worker"
account_id = "<account id here>"
route = "staging.example.com/*"
workers_dev = false

kv-namespaces = [
    { binding = "MY_KV", id = "06779da6940b431db6e566b4846d64db" }
]

[env.production]
route = "example.com/*"
kv-namespaces = [
    { binding = "MY_KV", id = "07bc1f3d1f2a4fd8a45a7e026e2681c6" }
]
```

With the wrangler.toml above, you can specify `--env production` when you want to perform a KV action on the namespace `MY_KV` under `env.production`. For example, with the wrangler.toml above, you can get a value out of a production KV instance with:

```console
wrangler kv:key get --binding "MY_KV" --env=production "my key"
```

To learn more about environments, check out the [environments documentation](./environments.md).

## `kv:namespace`

### `create`

Creates a new namespace.

Takes an optional `--env` [environment](./environments.md) argument.

#### Usage

```console
$ wrangler kv:namespace create "MY_KV"
🌀  Creating namespace with title "worker-MY_KV"
✨  Success: WorkersKvNamespace {
    id: "e29b263ab50e42ce9b637fa8370175e8",
    title: "worker-MY_KV",
}
✨  Add the following to your wrangler.toml:
kv-namespaces = [
         { binding = "MY_KV", id = "e29b263ab50e42ce9b637fa8370175e8" }
]
```

### `list`

Outputs a list of all KV namespaces associated with your account id.

#### Usage
The example below uses the `jq` command line tool to pretty-print output.

```console
$ wrangler kv:namespace list | jq '.'
[
    {
        "id": "06779da6940b431db6e566b4846d64db",
        "title": "TEST_NAMESPACE"
    },
    {
        "id": "32ac1b3c2ed34ed3b397268817dea9ea",
        "title": "STATIC_CONTENT"
    }
]
```

### `delete`

Deletes a given namespace.

Requires `--binding` or `--namespace-id` argument.

Takes an optional `--env` [environment](./environments.md) argument.

#### Usage

```console
$ wrangler kv:namespace delete --binding=MY_KV
Are you sure you want to delete namespace f7b02e7fc70443149ac906dd81ec1791? [y/n]
yes
🌀  Deleting namespace f7b02e7fc70443149ac906dd81ec1791
✨  Success
```

## `kv:key`

### `put`

Writes a single key/value pair to the given namespace.

Requires `--binding` or `--namespace-id` argument.

Optional params include:

1. `--env`: The [environment](./environments.md) argument.
1. `--ttl`: Number of seconds for which the entries should be visible before they expire. At least 60. Takes precedence over 'expiration' option.
1. `--expiration`: Number of seconds since the UNIX epoch, indicating when the key-value pair should expire.
1. `--path`: Read value from the file at a given path. *This is good for security-sensitive operations, like uploading keys to KV; uploading from a file prevents a key value from being saved in areas like your terminal history.*

#### Usage

```console
$ wrangler kv:key put --binding=MY_KV "key" "value" --ttl=10000
✨  Success
```

```console
$ wrangler kv:key put --binding=MY_KV "key" value.txt --path
✨  Success
```

### `list`

Outputs a list of all keys in a given namespace.

Requires `--binding` or `--namespace-id` argument.

Optional params include:

1. `--env`: The [environment](./environments.md) argument.
1. `--prefix`: A prefix to filter listed keys.

#### Usage

The example below uses the `jq` command line tool to pretty-print output.

```console
$ wrangler kv:key list --binding=MY_KV --prefix="public" | jq '.'
[
    {
        "name": "public_key"
    },
    {
        "name": "public_key_with_expiration",
        "expiration": "2019-09-10T23:18:58Z"
    }
]
```

### `get`

Reads a single value by key from the given namespace.

Requires `--binding` or `--namespace-id` argument.

Takes an optional `--env` [environment](./environments.md) argument.

#### Usage

```console
$ wrangler kv:key get --binding=MY_KV "key"
value
```

### `delete`

Removes a single key value pair from the given namespace.

Requires `--binding` or `--namespace-id` argument.

Takes an optional `--env` [environment](./environments.md) argument.

#### Usage

```console
$ wrangler kv:key delete --binding=MY_KV "key"
Are you sure you want to delete key "key"? [y/n]
yes
🌀  Deleting key "key"
✨  Success
```

## `kv:bulk`

### `put`

Requires `--binding` or `--namespace-id` argument.

Writes a file full of key/value pairs to the given namespace. Takes as an argument a JSON file with a list of key-value pairs to upload (see JSON spec above). An example of JSON input:

```json
[
    {
        "key": "test_key",
        "value": "test_value",
        "expiration_ttl": 3600
    }
]
```

The schema below is the full schema for key-value entries uploaded via the bulk API:

| **Name**                       | **Description**                                              | Optional |
| ------------------------------ | ------------------------------------------------------------ | -------- |
| `key`<br />(String)            | A key's name. The name may be at most 512 bytes. All printable, non-whitespace characters are valid. | no       |
| `value`<br />(String)          | A UTF-8 encoded string to be stored, up to 2 MB in length.   | no       |
| `expiration`<br />(Number)     | The time, measured in number of seconds since the UNIX epoch, at which the key should expire. | yes      |
| `expiration_ttl`<br />(Number) | The number of seconds for which the key should be visible before it expires. At least 60. | yes      |
| `base64`<br />(Boolean)        | Whether or not the server should base64 decode the value before storing it. Useful for writing values that wouldn't otherwise be valid JSON strings, such as images. Defaults to `false` | yes      |

If both `expiration` and `expiration_ttl` are specified for a given key, the API will prefer `expiration_ttl`.

The `put` command also takes an optional `--env` [environment](./environments.md) argument.

#### Usage

```console
$ wrangler kv:bulk put --binding=MY_KV allthethingsupload.json
✨  Success
```

### `delete`

Requires `--binding` or `--namespace-id` argument.

Deletes all specified keys within a given namespace.
Takes as an argument a JSON file with a list of key-value pairs to delete (see JSON spec above). An example of JSON input:

```json
[
    {
        "key": "test_key",
        "value": "test_value",
        "expiration_ttl": 3600
    }
]
```

The `delete` command also takes an optional `--env` [environment](./environments.md) argument.

#### Usage

```console
$ wrangler kv:bulk delete --binding=MY_KV allthethingsdelete.json
Are you sure you want to delete all keys in allthethingsdelete.json? [y/n]
yes
✨  Success
```
