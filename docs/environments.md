# Environments

Environments is a feature that allows you to deploy the same project to multiple places under multiple names.

## Usage

A common use case that this feature enables is deploying your worker to a staging subdomain before pushing your worker to production. The top-level configuration will be used when running `wrangler publish`. In addition, you can specify multiple environments that you would like to deploy code to. Each environment can inherit properties from the configuration at the top level. `type` will always be inherited from the top-level configuration, you cannot specify different types for different environments. `name` is inherited and modified if left out of the environment configuration, a worker named `my-worker` with an environment `[env.dev]` would become `my-worker-dev`. Fields that can be inherited from the top level configuration are `account_id`, `zone_id`, and `webpack_config`. The rest of the fields must be defined per environment and will not be inherited.

### Examples

#### Base case - routes

This `wrangler.toml` has no environments defined and will publish `my-worker` to `example.com/*`

```toml
type = "webpack"
name = "my-worker"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"
workers_dot_dev = false # this field specifies that the worker should not be deployed to workers.dev
```

#### Base case - workers.dev

This `wrangler.toml` has no environments defined and will publish `my-worker` to `my-worker.subdomain.workers.dev`

```toml
type = "webpack"
name = "my-worker"
account_id = "12345678901234567890"
workers_dot_dev = true # this field specifies that the worker should be deployed to workers.dev
```

#### Adding Environments

This `wrangler.toml` adds two environments to the base case.

```toml
type = "webpack"
name = "my-worker-dev"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "dev.example.com/*"

[env.production]
name = "my-worker"
route = "dev.example.com/*"

[env.staging]
name = "my-worker-staging"
route = "staging.example.com/*"
```

In order to use environments with this configuration, you can pass the name of the environment via the `--env` flag.

With this configuration, Wrangler will behave in the following manner:

`wrangler publish` will publish your worker to the `dev.example.com/*` route.
`wrangler publish --env staging` will publish your worker to the `staging.example.com/*`.
`wrangler publish --env production` will publish your worker to the `example.com/*` route.

#### workers.dev Environment

In order to deploy your code to workers.dev, you must include `workersdotdev = true` in the desired environment. Your `wrangler.toml` may look like this:

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"

[env.staging]
workersdotdev = true
```

With this configuration, Wrangler will behave in the following manner:

`wrangler publish` will publish your project to `example.com/*`
`wrangler publish --environment staging` will publish your project to `my-worker-staging.yoursubdomain.workers.dev`

## Invalid configurations

##### Multiple types

You cannot specify a type for each environment, type must be specified at the top level of `wrangler.toml`. 

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"

[env.staging]
type = "rust"
workersdotdev = true
```

##### Same name for multiple environments

You cannot specify multiple environments with the same name. If this were allowed, publishing each environment would overwrite your previously deployed worker, and the behavior would not be clear.

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"

[env.staging]
name = "my-worker"
workersdotdev = true
```

##### Ambiguous top level configuration

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*
```

You will be warned if `workers_dot_dev` is left out of the top level configuration because if it is not specified, it is unclear what the behavior of `wrangler publish` should be. See [the section on backwards compatibility](#Backwards-compatibility) for more information.

##### Defining workers_dot_dev and route

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*

[env.staging]
workers_dot_dev = true
route = "staging.example.com/*"
```

You will be warned if you publish to an environment where `route` is defined and `workers_dot_dev`. Wrangler will publish to `workers.dev`. The reason for the warning is because it is unclear what the intended behavior of the environment should be. Wrangler will assume you mean to deploy to `workers.dev`.

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

`wrangler publish` will publish your worker to your subdomain on workers.dev
`wrangler publish --release` will publish your worker to your route at `example.com/*`.

It is important to note that both of these commands will issue a deprecation warning. To remove these warnings, you can configure Wrangler with environments to separate deploys to workers.dev from deploys to workers routes.
