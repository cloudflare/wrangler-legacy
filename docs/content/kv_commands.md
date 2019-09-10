# 🗂️ `kv`

## Overview

The `kv` subcommand allows you to store application data in the Cloudflare network to be accessed from Workers. KV operations are scoped to your account, so in order to use any of these commands, you need to:

* have a Wrangler project set up with your `account_id` configured in the `wrangler.toml`
* call commands from within a Wrangler project directory.

Most `kv` subcommands allow you to specify an `--env`, or environment, variable. This allows you to gate
distinct namespaces within their own environments. For example, you could use seperate staging and production
namespaces for KV data in your wrangler.toml:
```toml
type = "webpack"
name = "my-worker"
account_id = "<account id here>"
route = "staging.example.com/*"
workers_dot_dev = false

kv-namespaces = [
	{ binding = "KV", id = "06779da6940b431db6e566b4846d64db" }
]

[env.production]
route = "example.com/*"
kv-namespaces = [
	{ binding = "KV", id = "07bc1f3d1f2a4fd8a45a7e026e2681c6" }
]
```

With the wrangler.toml above, you can specify `--env production` when you want to perform a KV action on
the namespace `KV` under `env.production`. To learn more about environments, check out the [environments documentation](./environments).

## `kv:namespace`

### `create`

Creates a new namespace.

#### Usage

```sh
$ wrangler kv:namespace create "new kv namespace"
🌀  Creating namespace with title "new kv namespace"
✨  Success: WorkersKVNamespace {
    id: "f7b02e7fc70443149ac906dd81ec1791",
    title: "new kv namespace",
}
```

### `delete`
Deletes a given namespace.
Takes an optional `--env` [environment](./environments) argument.

#### Usage

```sh
$ wrangler kv:namespace delete MY_NAMESPACE
Are you sure you want to delete namespace f7b02e7fc70443149ac906dd81ec1791? [y/n]
yes
🌀  Deleting namespace f7b02e7fc70443149ac906dd81ec1791
✨  Success
```

### `rename`
Renames a given namespace.
Takes an optional `--env` [environment](./environments) argument.

#### Usage

```sh
$ wrangler kv:namespace rename OLD_NAMESPACE NEW_NAMESPACE
🌀  Renaming namespace f7b02e7fc70443149ac906dd81ec1791 with title "NEW_NAMESPACE"
✨  Success
```

### `list`

Outputs a list of all KV namespaces associated with your account id.

#### Usage

```sh
$ wrangler kv:namespace list
🌀  Retrieving namespaces
✨  Success:
+---------------+----------------------------------+
| TITLE         | ID                               |
+---------------+----------------------------------+
| NEW_NAMESPACE | f7b02e7fc70443149ac906dd81ec1791 |
+---------------+----------------------------------+
```

## `kv:key`

### `put`

Writes a single key/value pair to the given namespace. Optional params include 
1. `--env`: The [environment](./environments) argument.
1. `--ttl`: Number of seconds for which the entries should be visible before they expire. At least 60. Takes precedence over 'expiration' option.
1. `--expiration`: Number of seconds since the UNIX epoch, indicating when the key-value pair should expire.
1. `--path`: Read value from the file at a given path. *This is good for security-sensitive operations, like uploading keys to KV; uploading from a file prevents a key value from being saved in areas like your terminal history.*


#### Usage

```sh
$ wrangler kv:key put MY_NAMESPACE "key" "value" --ttl=10000
✨  Success
```
```sh
$ wrangler kv:key put MY_NAMESPACE "key" value.txt --path
✨  Success
```

### `get`

Reads a single value by key from the given namespace.
Takes an optional `--env` [environment](./environments) argument.

#### Usage

```sh
$ wrangler kv:key get MY_NAMESPACE "key"
=> value
```

### `delete`

Removes a single key value pair from the given namespace.
Takes an optional `--env` [environment](./environments) argument.

#### Usage

```sh
$ wrangler kv:key delete MY_NAMESPACE "key"
Are you sure you want to delete key "key"? [y/n]
yes
🌀  Deleting key "key"
✨  Success
```

### `list`

Outputs a list of all keys in a given namespace. Optional params include
1. `--env`: The [environment](./environments) argument.
1. `--prefix`: A prefix to filter listed keys.

#### Usage
The example below uses Python's JSON pretty-printing command line tool to pretty-print output.

```sh
$ wrangler kv:key list MY_NAMESPACE --prefix="public" | python -m json.tool
[
    {
        "name": "public_key"
    }, 
    {
        "name": "public_key_with_expiration",
        "expiration": 1568014518
    } 
]
```

## `kv:bulk`

### `put`

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

The `put` command also takes an optional `--env` [environment](./environments) argument.

#### Usage

```sh
$ wrangler kv:bulk put MY_NAMESPACE allthethingsupload.json
✨  Success
```

### `delete`

Deletes all specified keys within a given namespace.
Takes as an argument a JSON file with a list of keys to delete; for example:
```json
[
    "key1",
    "key2"
]
```

The `delete` command also takes an optional `--env` [environment](./environments) argument.

#### Usage

```sh
$ wrangler kv:bulk delete MY_NAMESPACE allthethingsdelete.json
Are you sure you want to delete all keys in allthethingsdelete.json? [y/n]
yes
✨  Success
```

