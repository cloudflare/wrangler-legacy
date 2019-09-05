# Environments

Environments is a feature that allows you to deploy the same project to multiple places under multiple names.

## Usage

A common use case that this feature enables is deploying your worker to a staging subdomain before pushing your worker to production. The top-level configuration will be used when running `wrangler publish`. In addition, you can specify multiple environments that you would like to deploy code to. Each environment can inherit properties from the configuration at the top level. `type` will always be inherited from the top-level configuration, you cannot specify different types for different environments. `name` is inherited and modified if left out of the environment configuration, a worker named `my-worker` with an environment `[env.dev]` would become `my-worker-dev`. Fields that can be inherited from the top level configuration are `account_id`, `zone_id`, `workers_dot_dev`, and `webpack_config`. `kv_namespaces` and `route` must be defined for each environment and will not be inherited.

### Concepts

"top level configuration" refers to the configuration values you specify at the top of your `wrangler.toml`
"environment configuration" refers to the configuration values you specify under an `[env.name]` in your `wrangler.toml`

### Examples

#### Top level configuration

##### Routes

This `wrangler.toml` has no environments defined and will publish `my-worker` to `example.com/*`

```toml
type = "webpack"
name = "my-worker"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"
workers_dot_dev = false # this field specifies that the worker should not be deployed to workers.dev
```

```sh
$ wrangler publish
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at example.com/*
```

##### workers.dev

This `wrangler.toml` has no environments defined and will publish `my-worker` to `my-worker.subdomain.workers.dev`

```toml
type = "webpack"
name = "my-worker"
account_id = "12345678901234567890"
workers_dot_dev = true # this field specifies that the worker should be deployed to workers.dev
```

```sh
$ wrangler publish
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at https://my-worker.subdomain.workers.dev`
```

#### Introducing Environments

This `wrangler.toml` adds two environments to the base case.

```toml
type = "webpack"
name = "my-worker-dev"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "dev.example.com/*"
workers_dot_dev = false

[env.staging]
name = "my-worker-staging"
route = "staging.example.com/*"

[env.production]
name = "my-worker"
route = "example.com/*"
```

In order to use environments with this configuration, you can pass the name of the environment via the `--env` flag.

With this configuration, Wrangler will behave in the following manner:

```sh
$ wrangler publish
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at dev.example.com/*
```

```sh
$ wrangler publish --env staging
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at staging.example.com/*
```

```sh
$ wrangler publish --env production
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at example.com/*
```

#### Staging Environment with workers.dev

In order to deploy your code to workers.dev, you must include `workers_dot_dev = true` in the desired environment. Your `wrangler.toml` may look like this:

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"
workers_dot_dev = false

[env.staging]
workers_dot_dev = true
```

With this configuration, Wrangler will behave in the following manner:

```sh
$ wrangler publish
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at example.com/*
```

```sh
$ wrangler publish --env staging
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at https://my-worker-staging.subdomain.workers.dev
```

#### workers.dev as a first class target

If you only want to deploy to workers.dev you can configure Wrangler like so:

```toml
name = "my-worker-dev"
type = "webpack"
account_id = "12345678901234567890"
workers_dot_dev = true

[env.production]
name = "my-worker"

[env.staging]
name = "my-worker-staging"
```

With this configuration, Wrangler will behave in the following manner:

```sh
$ wrangler publish
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at https://my-worker-dev.subdomain.workers.dev
```

```sh
$ wrangler publish --env staging
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at https://my-worker-staging.subdomain.workers.dev
```

```sh
$ wrangler publish --env production
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at https://my-worker.subdomain.workers.dev
```

## Invalid configurations

### Multiple types

You cannot specify a type for each environment, type must be specified at the top level of your `wrangler.toml`.

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"
workers_dot_dev = true

[env.staging]
type = "rust"
```

Wrangler will not error with this configuration, it will build with the `webpack` type.

### Same name for multiple environments

You cannot specify multiple environments with the same name. If this were allowed, publishing each environment would overwrite your previously deployed worker, and the behavior would not be clear.

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"
workers_dot_dev = false

[env.staging]
name = "my-worker"
workers_dot_dev = true
```

```sh
$ wrangler publish
Error: ⚠️  Each name in your `wrangler.toml` must be unique, this name is duplicated: my-worker
```

```sh
$ wrangler publish --env staging
Error: ⚠️  Each name in your `wrangler.toml` must be unique, this name is duplicated: my-worker
```

### Ambiguous top level configuration

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*
```

You will be warned if `workers_dot_dev` is left out of the top level configuration because if it is not specified, it is unclear what the behavior of `wrangler publish` should be. See [the section on backwards compatibility](#Backwards-compatibility) for more information.

```sh
$ wrangler publish
⚠️  Please specify the workers_dot_dev boolean in the top level of your wrangler.toml
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at https://my-worker.avery.workers.dev
```

### Defining workers_dot_dev and route

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*
workers_dot_dev = true

[env.staging]
workers_dot_dev = true
route = "staging.example.com/*"
```

Wrangler will fail to publish to an environment where `route` is defined alongside `workers_dot_dev = true`.

```sh
$ wrangler publish
Error: ⚠️  Your environment should only include `workers_dot_dev` or `route`
```

```sh
$ wrangler publish --env staging
Error: ⚠️  Your environment should only include `workers_dot_dev` or `route`
```

## Backwards compatibility

Legacy `wrangler.toml` files will still work as expected during the initial rollout of this feature, however you will notice warnings when your configuration is ambigious. One of the goals of environments is to make it more obvious when you are deploying to a traditional worker with routes, and when you are deploying to a subdomain on workers.dev.

A `wrangler.toml` before this release looks like this:

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
private = false
route = "example.com/*
```

With this configuration, Wrangler will behave in the following manner:

```sh
$ wrangler publish
⚠️  Please specify the workers_dot_dev boolean in the top level of your wrangler.toml
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at https://my-worker.subdomain.workers.dev
```

```sh
$ wrangler publish --release
⚠️  --release will be deprecated
⚠️  Please specify the workers_dot_dev boolean in the top level of your wrangler.toml
✨  Built successfully, built project size is 517 bytes.
✨  Successfully published your script.
✨  Success! Your worker was successfully published. You can view it at example.com/*
```

This backwards compatibility is the reason that a warning is thrown if `workers_dot_dev` is not specified at the top of `wrangler.toml`.

It is important to note that both of these commands will issue a deprecation warning. To remove these warnings, you can configure Wrangler with the `workers_dot_dev` boolean to separate deploys to workers.dev from deploys to workers routes.
