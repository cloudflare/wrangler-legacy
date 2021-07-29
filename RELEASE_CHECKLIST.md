# Release Checklist

This is a list of the things that need to happen during a release.

## Build a Release

### Prepare the Changelog (Full release only)

1. Open the associated milestone, if it exists. All issues and PRs should be closed. If
   they are not you should reassign all open issues and PRs to future
   milestones.
1. Run the changelog generator in `./changelog-generator`.
   1. Get the date after the most recent release,
   1. run `cd ./changelog-generator && npm install && node index.js cloudflare wrangler YYYY-MM-DD`
   1. the generated changelog is in `./changelog-generator/output.md`. Open it, and add the version at the top, and move the entries to their proper category.
1. Choose an emoji for the release. Try to make it semi-related to something that's been included in the release (point releases can be a little weirder).
1. Add the contents of `output.md` to the top of `CHANGELOG.md`, matching the structure of previous
   entries. If it is a release candidate, no official changelog is needed, but testing instructions will be added later in the process.

### Update cargo manifest

1. Update the version in `Cargo.toml`.
1. Run `cargo update`.
1. Run `cargo test`.
1. Run `cargo build`.

### Update npm manifest and assets

1. Copy `README.md` to `npm/README.md`
1. Bump the version number in `npm/package.json`
1. `cd npm && npm install` _Note: This step will appear to fail due to the new version not existing yet, however its utility is re-building npm-shrinkwrap.json_

### Start a release PR

1. Create a new branch "#.#.#" where "#.#.#" is this release's version (release) or "#.#.#-rc.#" (release candidate)
1. Push up a commit with the `Cargo.toml`, `Cargo.lock`, `npm/README.md`, `npm/package.json`, `npm/npm-shrinkwrap.json`,
   and `CHANGELOG.md` changes. The commit message can just be "#.#.#" (release) or "#.#.#-rc.#" (release candidate)
1. Request review from the @cloudflare/workers-devexp team.

### Review

Most of your comments will be about the changelog. Once the PR is finalized and approved...

1. If you made changes, squash or fixup all changes into a single commit.
1. Run `git push` and wait for CI to pass.
## Merge

1. Hit the big green Merge button on the release PR.
1. `git checkout master` and `git pull --rebase origin master`

### Tag and build release

This part of the release process is handled by GitHub Actions, and our binaries are distributed as GitHub Releases. When you push a version tag, it kicks off an action that creates a new GitHub release for that tag, builds release binaries and attaches them to the release.

1. After pulling `master` in the step above, tag the commit by running either `git tag -a v#.#.# -m #.#.#` (release), or `git tag -a v#.#.#-rc.# -m #.#.#` (release candidate)
1. Run `git push --tags`.
1. Wait for CI to pass.
1. If CI fails, delete the tag locally and remotely
1. Fix whatever caused the CI failure
1. Re-tag the healthy commit, and wait for CI to pass again.

### Edit the release

After CI builds the release binaries and they appear on the [releases page](https://github.com/cloudflare/wrangler/releases), click `Edit` and update release notes.

#### For Full Releases

1. Paste the current release notes from `CHANGELOG.md` into the release body.
1. Update the *title* of the release (not the tag itself) to include the emoji for the current release
1. Be sure to add any missing link definitions to the release.

#### For Release Candidates

1. Mark the release as a `pre-release`. This is handled with a checkbox on the Edit page.
1. If this is a new rc (rc.0), paste testing instructions into the release notes.
1. If this is a rc.1 or later, the old release candidate testing instructions should be moved to the latest release candidate testing instructions, and replaced with the following message:

   ```markdown
   This beta release is now out of date. If you previously installed this release, you should reinstall with `npm i -g @cloudflare/wrangler@beta` and see what's changed in the latest [release](https://github.com/cloudflare/wrangler/releases).
   ```

   The new release candidate should then include updated testing instructions with a small changelog at the top to get folks who installed the old release candidate up to speed.

### Publish to crates.io (full release only)

**IMPORTANT: This step is the hardest to fix if you mess it up. Do not run this step for Release Candidates**.

We don't publish release candidates to crates.io because they don't (as of this writing) have a concept of a "beta" version.

1. Run `cargo test`
1. (Release only) `cargo publish`

### Publish to npm

Full releases are tagged `latest`. Release candidates are tagged `beta`. If for some reason you mix up the commands below, follow the troubleshooting guide.

1. If this is a full release, `cd npm && npm publish`. If it is a release candidate, `cd npm && npm publish --tag beta`
1. Tweet.

# Troubleshooting a release

Mistakes happen. Most of these release steps are recoverable if you mess up. The goal is not to, but if you find yourself cursing a fat fingered command, here are some troubleshooting tips. Please feel free to add to this guide.

## I pushed the wrong tag

Tags and releases can be removed in GitHub. First, [remove the remote tag](https://stackoverflow.com/questions/5480258/how-to-delete-a-remote-tag):

``` console
$ git push --delete origin tagname
```

This will turn the release into a `draft` and you can delete it from the edit page.

Make sure you also delete the local tag:

``` console
$ git tag --delete vX.X.X
```

## I forgot to add the `beta` tag to my RC when I ran `npm publish`

Never fear! We can fix this by updating npm tags. First, add a beta tag for the version you just published:

``` console
$ npm dist-tag add @cloudflare/wrangler@x.x.x-rc.x beta
```

once you add the beta tag, pause...

...and then list your tags:

``` console
$ npm dist-tag ls @cloudflare/wrangler
```

You should now see two tags pointing to the version you just pushed; for example if you had tried to push v1.9.0-rc.0:

``` console
$ npm dist-tag ls @cloudflare/wrangler
beta: 1.9.0-rc.0
latest: 1.9.0-rc.0
```

Go back to the Changelog or GitHub releases, find the _actual_ latest version, and re-tag it as latest:

``` console
$ npm dist-tag add @cloudflare/wrangler@x.x.x latest
```

List tags again and you should see the latest restored, and your new release candidate as beta (e.g. 1.9.0-rc.0 is beta and 1.8.4 was last stable version)

``` console
$ npm dist-tag ls @cloudflare/wrangler
beta: 1.9.0-rc.0
latest: 1.8.4
```
