For each of the Wrangler projects in this directory, the following procedure should result in a successful write to a KV namespace in the user's project.

## Setup: do once for all tests

### Set env vars

`test_env.sh.sample` contains all the env var keys you should need to run these tests. `cp` it to `test_env.sh` and run `source test_env.sh` before testing to pull those keys into your environment during testing.

### Add test KV namespace

Before running any of these tests, you should have a Cloudflare account with entitlement for KV. You should also have configured a subdomain for your Cloudflare account.

Run the following curl command to add the appropriate KV namespace, substituting your Cloudflare Account ID, Auth Email, and Auth Key. It's a good idea to export these as environment variables if you find yourself repeatedly running these commands.

``` sh
curl -X POST "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/storage/kv/namespaces" \
     -H "X-Auth-Email: $CLOUDFLARE_AUTH_EMAIL" \
     -H "X-Auth-Key: $CLOUDFLARE_AUTH_KEY" \
     -H "Content-Type: application/json" \
     --data '{"title":"test kv integration"}'
```

The output of this request will look like the following:

``` sh
{
  "result": {
    "id": "a99213f3975246aca6b83dec10873c97",
    "title": "test kv integration"
  },
  "success": true,
  "errors": [],
  "messages": []
}
```

Record the `result.id` field for use in the wrangler.toml files. Consider also adding it as an environment variable `$NAMESPACE_ID` to aid in this process.

#### Error: namespace already exists

If you receive an error that looks like this:

``` sh
{
  "result": null,
  "success": false,
  "errors": [
    {
      "code": 10014,
      "message": "a namespace with this account ID and title already exists"
    }
  ],
  "messages": []
}
```

Run the following to get a list of your namespaces.

``` sh
curl -X GET "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/storage/kv/namespaces" \
     -H "X-Auth-Email: $CLOUDFLARE_AUTH_EMAIL" \
     -H "X-Auth-Key: $CLOUDFLARE_AUTH_KEY"
```

And find the entry with title "test kv integration".

### Set a value in your new namespace.

Run the following to set the KV pair "foo: bar" on the test namespace, subbing in your auth values and the NAMESPACE_ID you retrieved in the Create KV Namespace step:

``` sh
curl -X PUT "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/storage/kv/namespaces/$NAMESPACE_ID/values/foo" \
     -H "X-Auth-Email: $CLOUDFLARE_AUTH_EMAIL" \
     -H "X-Auth-Key: $CLOUDFLARE_AUTH_KEY" \
     -H "Content-Type: text/plain" \
     --data 'bar'
```

This value will take about ten seconds to populate to the edge, so keep this in mind when automating these tests. The same key/value pair is used across tests.

### Configure Wrangler

Run `cargo run -- config` with your Cloudflare auth email and auth key.

## Test

### Best Case - Successful Namespace integration

#### Step 1: Update wrangler.toml

In each project, add your account id and the newly generated kv namespace id to the toml (both of these should be strings):

``` toml
name = "webpack-worker"
type = "webpack"
private = false
account_id = <YOUR ACCOUNT ID>

[[kv-namespaces]]
binding = "TEST_KV_INTEGRATION"
id = "NEW KV NAMESPACE ID"
```

#### Step 2: Publish the Worker

From the project root run the following Wrangler command and wait for it to complete:

``` sh
cargo run -- publish
```

The publish command should exit successfully and return the URL to use in Step 3.

#### Step 3: Call the Worker

Run the following `curl` command, substituting your workers.dev subdomain:

``` sh
curl -X GET "https://test-worker.$YOUR_SUBDOMAIN.workers.dev"
```

The response should include the value you added to the KV store in the setup stage (in this case "bar").

### Missing KV Namespace fields

Follow step 1 and 2 with the following variations on the `wrangler.toml` configurations in Step 1. Step 2 should error out with the indicated error messages:

* no binding value => "mis-configured KV Namespace: missing binding"
* binding value of empty string => "mis-configured KV Namespace: missing binding"
* no namespace id => "mis-configured KV Namespace: missing id"
* namespace id of empty string => "mis-configured KV Namespace: missing id"

### Invalid KV Namespace Binding

Follow step 1 and 2 with the following variations on the `wrangler.toml` configurations in Step 1. Step 2 should error out with the indicated error messages:

* invalid binding value => "mis-configured KV Namespace: binding must be a valid JavaScript symbol"

### KV Namespace does not exist

*Note* This last test is a stretch goal; the API does not currently return a specific error when this is the case, just a generic 400. This "feature" is blocked by https://jira.cfops.it/browse/EW-2087.

Follow step 1 and 2 with the following variations on the `wrangler.toml` configurations in Step 1. Step 2 should error out with the indicated error messages:

* invalid id value => "mis-configured KV Namespace: id not found"

## Clean up

Run the following three `curl` commands to clean up your namespace and your worker once you've finished running tests:

``` sh
curl -X DELETE "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/storage/kv/namespaces/$NAMESPACE_ID/values/foo" \
     -H "X-Auth-Email: $CLOUDFLARE_AUTH_EMAIL" \
     -H "X-Auth-Key: $CLOUDFLARE_AUTH_KEY"

curl -X DELETE "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/storage/kv/namespaces/$NAMESPACE_ID" \
     -H "X-Auth-Email: $CLOUDFLARE_AUTH_EMAIL" \
     -H "X-Auth-Key: $CLOUDFLARE_AUTH_KEY"

curl -X DELETE "https://api.cloudflare.com/client/v4/accounts/$CLOUDFLARE_ACCOUNT_ID/workers/scripts/$SCRIPT_NAME" \
     -H "X-Auth-Email: $CLOUDFLARE_AUTH_EMAIL" \
     -H "X-Auth-Key: $CLOUDFLARE_AUTH_KEY"
```