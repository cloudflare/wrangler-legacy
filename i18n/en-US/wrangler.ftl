### Localization terms for `wrangler`
###
### Syntax: https://projectfluent.org/fluent/guide/
### Playground: https://projectfluent.org/play/

## Terms

-cli = wrangler

## Sentences

hello-user = Hello {$name}, welcome to {-cli}!
hello-script = You have { $count ->
    [0]     no scripts.
    [1]     1 script.
    *[other] { $count } scripts.
}
