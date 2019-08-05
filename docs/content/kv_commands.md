# 🗂️ `kv`

## Overview

The `kv` subcommand allows you to store application data in the Cloudflare network to be accessed from Workers. KV operations are scoped to your account, so in order to use any of these commands, you need to:

* have a Wrangler project set up with your `account_id` configured in the `wrangler.toml`
* call commands from within a Wrangler project directory.

<!-- TODO: add gif of `wrangler generate` through `wrangler kv create` -->

## Commands

### ✨ `create <namespace-title>`

Creates a new namespace.

#### Usage

``` sh
$ wrangler kv create "new kv namespace"
🌀  Creating namespace with title "new kv namespace" 🌀 
✨  Success: WorkersKVNamespace {
    id: "f7b02e7fc70443149ac906dd81ec1791",
    title: "new kv namespace",
}
```
