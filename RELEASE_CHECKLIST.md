# Release Checklist

This is a list of the things that need to happen during a release.

1. Open the associated milestone. All issues and PRs should be closed. If
   they are not you should reassign all open issues and PRs to future
   milestones.
1. Go through the commit history since the last release. Ensure that all PRs
   that have landed are marked with the milestone. You can use this to
   show all the PRs that are merged on or after YYY-MM-DD:
   `https://github.com/issues?utf8=%E2%9C%93&q=repo%3Acloudflare%2Fwrangler+merged%3A%3E%3DYYYY-MM-DD`
1. Go through the closed PRs in the milestone. Each should have a changelog
   label indicating if the change is docs, fix, feature, or maintenance. If
   there is a missing label, please add one.
1. Choose an emoji for the release. Try to make it semi-related to something that's been included in the release (point releases can be a little weirder).
1. Create a new branch "#.#.#" where "#.#.#" is this release's version.
1. Add this release to the `CHANGELOG.md`. Use the structure of previous
   entries. If you use VS Code, you can use [this snippet](https://gist.github.com/EverlastingBugstopper/04d1adb99506388ff9d7abd8d0a82bc3) to insert new changelog sections.
1. Update the version in `Cargo.toml`.
1. Run `cargo update`.
1. Run `cargo test`.
1. Run `cargo build`.
1. Copy `README.md` to `npm/README.md`
1. Bump the version number in `npm/package.json`
1. `cd npm && npm shrinkwrap`
1. Push up a commit with the `Cargo.toml`, `Cargo.lock`,
   and `CHANGELOG.md` changes. The commit message can just be "#.#.#".
1. Request review from the @cloudflare/workers-devexp team.
1. `git commit --amend` all changes into a single commit.
1. Once ready to merge, tag the commit by running `git tag -a v#.#.# -m #.#.#`.
1. Wait for CI to pass.
1. After CI builds the release binaries and they appear on the [releases page](https://github.com/cloudflare/wrangler/releases), click `Edit`, and
   paste the current release notes from `CHANGELOG.md` and paste it into the release body.
1. Update the title of the release (not the tag itself) to include the emoji for the current release
1. Be sure to add any missing link definitions to the release.
1. Hit the big green Merge button on the release PR.
1. `git checkout master` and `git pull --rebase origin master`
1. Run `cargo test`.
1. `cargo publish`
1. `cd npm && npm publish`
1. Tweet.
