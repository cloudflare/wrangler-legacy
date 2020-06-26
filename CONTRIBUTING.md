# Contributing

Wrangler is an open source project because we believe that folks should have access, insight,
and the opportunity to contribute to their developer tools. Wrangler is also a product
delivered by Cloudflare, so it's important to clarify how we think about issue triage and
contributions.

If you want to learn about how to get started developing Wrangler, [click here](#Developing-Wrangler)

## People

Wrangler is owned by the [Cloudflare Workers](https://workers.cloudflare.com) Team and maintained by community members like you! The core maintainers are @EverlastingBugstopper and @ashleymichal, and everything that gets merged must be approved by at least one team member.

## Developing Wrangler

### Get started

To get started with developing Wrangler, we recommend that you first get [up to speed with Wrangler](https://developers.cloudflare.com/workers/quickstart). Then, get up to speed with the [basics of Rust](https://www.rust-lang.org/learn/get-started). (You'll need to install `rustup` where we're going).

### Build Wrangler from source

To build Wrangler from source, fork the repo, clone it, and `cd wrangler`. To run the test suite, run `cargo test`. To quickly test a command, for example  `wrangler init`, run `cargo run -- init`. This will compile a "debug" binary, then run it, passing in all arguments after `--`.

It is important to note that many Wrangler commands depend on a `wrangler.toml` in the current working directory. It is advised to keep a test project somewhere with an active `wrangler.toml` to exercise your code.

### Run Wrangler during development

There are many ways to build and execute a development version of Wrangler, in addition to using `cargo run`:

`cargo build` will produce a local binary at `./target/debug/wrangler` that you can execute just by calling it by path.
`cargo install --debug --path .` will replace any globally installed wrangler with the one you've just built from source.

You can read more about cargo [here](https://doc.rust-lang.org/cargo/), just find something that works for you.

### Module System

Each of the commands supported by Wrangler have entrypoints in [./src/commands](./src/commands). It's useful to understand the [module system](https://doc.rust-lang.org/rust-by-example/mod.html) if you will be adding new commands or need to refactor/organize imports.

### Notable external libraries

#### Command-line argument parsing (clap)

The primary framework we use for developing Wrangler is called [clap](https://clap.rs), which provides fast and structured argument parsing. This is how different features are exposed to users, and most of that logic lives in [main.rs](./src/main.rs).

#### API calls (cloudflare-rs)

When developing a new feature for Wrangler, it's quite common to need to make API calls. The way we do this is by submitting a PR to [cloudflare-rs](https://github.com/cloudflare/cloudflare-rs) and releasing a new version of that library to depend on. There are some legacy endpoints that we use our own client for, but the goal is to eventually move everything to cloudflare-rs. All endpoint calls should be made with the clients in [./src/http](./src/http).

### Figure out where to start

If you're working on a specific issue, make sure there is buy-in from the Wrangler team before starting, and feel free to ask where you should start. We're more than happy to help!

### Requirements for merging a PR

### Passing tests

When adding features or fixing bugs, we'd love if you would add a test! There are two types of tests in Wrangler, integration and unit tests. To execute tests, you can run `cargo test`. You can read more about testing [here](https://doc.rust-lang.org/rust-by-example/testing.html). All tests must pass when submitting a new PR, this is enforced by our GitHub Actions runners, which run each test on Windows, MacOS, and Linux.

### Proper formatting

You must run `cargo fmt` on your code before CI will allow you to merge your PR.

## Primary Issue Triage

Within 3 days, any incoming issue should be triaged. Triage involves:

- reading the issue and requesting any further information
- always thank the user for submitting
- assigning appropriate labels

### Labelling

- each issue should have a `status` label and a `category` label, and they should be kept up to date
  - once design has been settled for an issue, please label with `status - PR welcome`
- each issue from non-team members should be labelled `user report` (issue templates assign this automatically)
- subject labels and other call to actions are nice to have
- if an issue seems easy to tackle, please label with `good first issue` so new contributors can use it to ramp up

### Assignment

- if the issue will require a large amount of back and forth between the reporter and the team
    assign a single team member to manage the conversation

## Product Issue Triage

Once a week, the team holds the Wrangler Contributors meeting. This is where we assign work and update
our plans for the milestones and releases.

### Assignment and Milestones

- assign all issues for the next two releases a milestone
- assign all issues for the current milestone a person to take point

## Pull Request Triage

Within 3 days, all incoming Community PRs should be triaged. If a team member opens a PR it
should be triaged immediately upon open by the PR author.

### Labelling

- All work-in-progress PRs should be labelled `work in progress` and the title should be
    annotated [WIP] for easy scanning. No WIP PRs will be reviewed until the annotations
    are removed.
- All PRs that need to be reviewed should be labelled `needs review` until they have
    received all required reviews.
- All PRs should be labelled with a changelog label: `BREAKING`, `feature`, `fix`, `maintenance`, `docs`, `N/A`
- All PRs that are ready for review should be tagged with the appropriate release milestone

### Merging

- All PRs should be merged with a Merge Commit. We recommend that folks rebase into a small
    number of task driven commits. This is enforced more heavily for team members than
    community members. Be reasonable.
- All PRs should be labelled with the current milestone before merging. If a PR for an issue
    labelled with a different milestone is to be merged, update the issue milestone as well.
