# Environments

Environments is a feature that allows you to deploy the same project to multiple places.

## Backwards compatibilty

Legacy `wrangler.toml` files will still work as expected during the initial rollout of this feature, however you may notice a deprecation warning. One of the goals of environments is to make it more obvious when you are deploying to a traditional worker with routes, and when you are deploying to a subdomain on workers.dev.

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

## Intended Use

A common use case that this feature enables is deploying your worker to a staging subdomain before pushing your worker to production. The top-level configuration will be used when running `wrangler publish`. In addition, you can specify multiple environments that you would like to deploy code to.

### Example

```toml
type = "webpack"
name = "my-worker"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"

[env.staging]
name = "my-worker-staging"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "staging.example.com/*"

[env.dev]
name = "my-worker-dev"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "dev.example.com/*"
```

In order to use environments with this configuration, you can pass the name of the environment via the `--environment` flag. 

With this configuration, Wrangler will behave in the following manner:

`wrangler publish` will publish your worker to the `example.com/*` route.
`wrangler publish --environment staging` will publish your worker to the `staging.example.com/*` route.
`wrangler publish --environment dev` will publish your worker to the `dev.example.com/*` route.

## Deploying to a workers.dev environment

In order to deploy your code to workers.dev, you must include `workersdotdev = true` in the desired environment. Your `wrangler.toml` may look like this:

```toml
name = "my-worker"
type = "webpack"
account_id = "12345678901234567890"
zone_id = "09876543210987654321"
route = "example.com/*"

[env.staging]
name = "staging"
private = false
account_id = "1234567890"
workersdotdev = true
```

With this configuration, Wrangler will behave in the following manner:

`wrangler publish` will publish your project to `example.com/*`
`wrangler publish --environment staging` will publish your project to `staging.yoursubdomain.workers.dev`
