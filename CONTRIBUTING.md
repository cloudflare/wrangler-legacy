# Contributing

Wrangler is an open source project because we believe that folks should have access, insight,
and the opportunity to contribute to their developer tools. Wrangler is also a product
delivered by Cloudflare, so it's important to clarify how we think about issue triage and
contributions.

## People

Wrangler is maintained by @ashleygwilliams, and her team, [Workers Developer Experience].

[Workers Developer Experience]: https://github.com/orgs/cloudflare/teams/workers-devexp

## Primary Issue Triage

Within 3 days, any incoming issue should be triaged. Triage involves:

- reading the issue and requesting any further information
- always thank the user for submitting

### Labelling

- label all issues coming from non-team members with User Report
- labelling the category of the issue: Feature, External Bug, Bug, Maintenance, Docs
- optionally labelling a secondary category: Webpack, Routes, Workers Runtime, Refactor
- labelling the status of the issue: Need More Info, Needs Repro, Needs Design, PR Welcome
- optionally labelling other calls to action: Help Wanted, Question

### Assignment

- if the issue will require a large amount of back and forth between the reporter and the team
    assign a single team member to manage the conversation

## Product Issue Triage

Once a week, the team meets to do Product Triage. This is where we assign work and update
our plans for the milestones and releases.

### Labelling

- labelling the priority of the issue: Critical, Nice to Have, Low Priority
- labelling the status of the issue: Needs Design, PR Welcome

### Assignment and Milestones

- assign all issues for the next two releases a milestone
- assign all issues for the current milestone a person to take point

### Pull Request Triage

Within 3 days, all incoming Community PRs should be triaged. If a team member opens a PR it
should be triaged immediately upon open by the PR author.

### Labelling

- All work-in-progress PRs should be labelled Work In Progress and the title should be
    annotated [WIP] for easy scanning. No WIP PRs will be reviewed until the annotations
    are removed.
- All PRs that need to be reviewed should be labelled Needs Review until they have 
    received all required reviews.
- All PRs should be labelled with a changelog label: BREAKING, Feature, Bug, Maintenance, Docs

### Merging

- All PRs should be merged with a Merge Commit. We recommend that folks rebase into a small
    number of task driven commits. This is enforced more heavily for team members than
    community members. Be reasonable.
- All PRs should be labelled with the current milestone before merging. If a PR for an issue
    labelled with a different milestone is to be merged, update the issue milestone as well.
