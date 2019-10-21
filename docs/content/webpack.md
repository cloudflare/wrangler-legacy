# 🦄 Webpack

Out of the box, Wrangler allows you to develop modern ES6 applications with support for modules. This is because of the 🧙‍♂️ magic of [webpack](https://webpack.js.org/). This document describes how Wrangler uses webpack to build your Workers, and how you can bring your own configuration.

**IMPORTANT: In order for Wrangler to use webpack to bundle your worker scripts, you must set `type = "webpack"` in your `wrangler.toml`, no other types will build your script with webpack.**

If you're here because you're seeing warnings about specifying `webpack_config`, click [here](#backwards-compatibility)

## Sensible Defaults

This is the default webpack configuration that Wrangler uses to build your worker:

```js
module.exports = {
  "target": "webworker",
  "entry": "./index.js" // inferred from "main" in package.json
};
```

Our default configuration sets `target` to `webworker`. From the [webpack docs](https://webpack.js.org/concepts/targets/):

> Because JavaScript can be written for both server and browser, webpack offers multiple deployment targets that you can set in your webpack configuration.

Cloudflare Workers are built to match the [Service Worker API](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API), so we set our `target` to `webworker`.

The `entry` field is taken directly from the `main` field in your `package.json`. To read more about the webpack `entry` property, click [here](https://webpack.js.org/concepts/entry-points/).

## Bring your own configuration

You can tell Wrangler to use a custom webpack configuration file by setting `webpack_config` in your `wrangler.toml`. You'll want to make sure that `target` is always `webworker`.

### Example

`webpack.config.js`

```js
module.exports = {
  "target": "webworker",
  "entry": "./index.js",
  "mode": "production"
}
```

`wrangler.toml`

```toml
type = "webpack"
name = "my-worker"
account_id = "12345678901234567890"
workers_dev = true
webpack_config = "webpack.config.js"
```

### Example with multiple environments

`wrangler.toml`

```toml
type = "webpack"
name = "my-worker-dev"
account_id = "12345678901234567890"
workers_dev = true
webpack_config = "webpack.development.js"

[env.staging]
name = "my-worker-staging"
webpack_config = "webpack.production.js"

[env.production]
name = "my-worker-production"
webpack_config = "webpack.production.js"
```

`webpack.development.js`

```js
module.exports = {
  "target": "webworker",
  "entry": "./index.js",
  "mode": "development"
}
```

`webpack.production.js`

```js
module.exports = {
  "target": "webworker",
  "entry": "./index.js",
  "mode": "production"
}
```

## Backwards Compatibility

If you are using a version of Wrangler before 1.6.0, worker projects will simply use any `webpack.config.js` that is in the root of your project. This is not always obvious, so we plan to require that you specify `webpack_config` in your `wrangler.toml` if you would like to use it. If you're seeing this warning and would like to use your `webpack.config.js`, simply add `webpack_config = "webpack.config.js"` to your wrangler.toml.

If you are using Workers Sites and want to specify your own webpack configuration, you will always need to specify this. By default, Wrangler will not assume the `webpack.config.js` at the root of your project is meant to be used for building your Worker.
