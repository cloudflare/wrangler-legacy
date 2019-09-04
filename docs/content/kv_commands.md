# 🗂️ `kv`

## Overview

The `kv` subcommand allows you to store application data in the Cloudflare network to be accessed from Workers. KV operations are scoped to your account, so in order to use any of these commands, you need to:

* have a Wrangler project set up with your `account_id` configured in the `wrangler.toml`
* call commands from within a Wrangler project directory.

## `kv:namespace`

### `create`

Creates a new namespace.

#### Usage

```sh
$ wrangler kv:namespace create "new kv namespace"
🌀  Creating namespace with title "new kv namespace" 🌀 
✨  Success: WorkersKVNamespace {
    id: "f7b02e7fc70443149ac906dd81ec1791",
    title: "new kv namespace",
}
```

### `delete`

#### Usage

```sh
$ wrangler kv:namespace delete f7b02e7fc70443149ac906dd81ec1791
🌀  Deleting namespace f7b02e7fc70443149ac906dd81ec1791 🌀 
✨  Success
```

### `rename`

#### Usage

```sh
$ wrangler kv:namespace rename f7b02e7fc70443149ac906dd81ec1791 "updated kv namespace"
🌀  Renaming namespace f7b02e7fc70443149ac906dd81ec1791 with title "updated kv namespace"
✨  Success
```

### `list`

Outputs a list of all KV namespaces associated with your account id.

#### Usage

```sh
$ wrangler kv:namespace list
🌀  Retrieving namespaces 🌀 
✨  Success:
+------------------+----------------------------------+
| TITLE            | ID                               |
+------------------+----------------------------------+
| new kv namespace | f7b02e7fc70443149ac906dd81ec1791 |
+------------------+----------------------------------+
```

## `kv:key`

### `put`

Writes a single key/value pair to the given namespace.

#### Usage

```sh
$ wrangler kv:key put f7b02e7fc70443149ac906dd81ec1791 "key" "value" --ttl=10000
```

### `get`

Reads a single value by key from the given namespace.

#### Usage

```sh
$ wrangler kv:key get f7b02e7fc70443149ac906dd81ec1791 "key"
```

### `delete`

Removes a single key value pair from the given namespace.

#### Usage

```sh
$ wrangler kv:key delete f7b02e7fc70443149ac906dd81ec1791 "key"
```

### `list`

Outputs a list of all KV namespaces associated with your account id.

#### Usage

```sh
$ wrangler kv:key list f7b02e7fc70443149ac906dd81ec1791 --prefix="public"
🌀  Retrieving keys 🌀 
✨  Success:
+------------------+----------------------------------+
| KEY              | EXPIRATION                       |
+------------------+----------------------------------+
| "key"            | Wed Aug 28 10:28:44 CDT 2019     |
+------------------+----------------------------------+
```

## `kv:bulk`

### JSON body

Bulk operations take as an argument a pre-built JSON file, which should be a list of objects with the following schema:

| **Name**                       | **Description**                                              | Optional |
| ------------------------------ | ------------------------------------------------------------ | -------- |
| `key`<br />(String)            | A key's name. The name may be at most 512 bytes. All printable, non-whitespace characters are valid. | no       |
| `value`<br />(String)          | A UTF-8 encoded string to be stored, up to 2 MB in length.   | no       |
| `expiration`<br />(Number)     | The time, measured in number of seconds since the UNIX epoch, at which the key should expire. | yes      |
| `expiration_ttl`<br />(Number) | The number of seconds for which the key should be visible before it expires. At least 60. | yes      |
| `base64`<br />(Boolean)        | Whether or not the server should base64 decode the value before storing it. Useful for writing values that wouldn't otherwise be valid JSON strings, such as images. Defaults to `false` | yes      |

If both `expiration` and `expiration_ttl` are specified for a given key, the API will prefer `expiration_ttl`.

### `put`

Writes a file full of key/value pairs to the given namespace. Takes as its argument a giant json with a list of keys to upload (see JSON spec).

#### Usage

```sh
$ wrangler kv:bulk put f7b02e7fc70443149ac906dd81ec1791 ./allthethings.json
```

### `delete`

Deletes all specified keys within a given namespace.

#### Usage

```sh
$ wrangler kv:bulk delete f7b02e7fc70443149ac906dd81ec1791 ./allthethings.json
```

## `kv:bucket`

### `upload`

Walks the given directory and runs a bulk upload, using the path to an asset as its `key` and the asset as its `value`.

#### Usage

```sh
$ wrangler kv:bucket upload f7b02e7fc70443149ac906dd81ec1791 ./public
```

### `delete`

Walks the given directory and runs a bulk delete, using the paths to assets as the `key`s to delete.

#### Usage

```sh
$ wrangler kv:bucket upload f7b02e7fc70443149ac906dd81ec1791 ./public
```

