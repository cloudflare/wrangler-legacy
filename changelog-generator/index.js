const { program } = require("commander");
const { Octokit } = require("@octokit/rest");
const Mustache = require("mustache");
const prettier = require("prettier");
const fs = require("fs");

Mustache.escape = (t) => t;

const generate = async (owner, repo, since) => {
  const octokit = new Octokit({
    userAgent: "changelog generator",
  });

  let prs = await getMergedPRs(octokit, owner, repo, since);

  console.log("Contributors: ");
  let seen_contributors = {};
  for (const pr of prs) {
    if (!(pr.user.login in seen_contributors)) {
      console.log(`${pr.user.login} - ${pr.author_association}`);
      seen_contributors[pr.user.login] = true;
    }
  }

  let pulls = [];
  for (const pr of prs) {
    const pull_slug = pr.html_url.match(/pull\/\d+/)[0];

    let data = {
      title: pr.title,
      pull_url: pr.html_url,
      pull_slug,
      author_url: pr.user.html_url,
      author_slug: pr.user.login,
    };

    if (!pr.user.login.includes("dependabot")) {
      if (pr.body && pr.body.length > 240) {
        data.body = pr.body.substr(0, 240) + "\n... truncated";
      } else {
        data.body = pr.body;
      }
    } else {
      data.author_slug = "dependabot";
      data.author_url = "https://dependabot.com/";
    }

    pulls.push(data);
  }

  pulls = pulls.sort((a, b) => {
    if (a.author_slug == "dependabot" && b.author_slug != a.author_slug) {
      return 1;
    }
    const titleA = a.title.toUpperCase();
    const titleB = b.title.toUpperCase();
    if (titleA < titleB) {
      return -1;
    }
    if (titleA > titleB) {
      return 1;
    }

    return 0;
  });

  let pulls_formatted = [];
  for (const pull of pulls) {
    const { body, ...rest } = pull;

    const formatted = Mustache.render(fs.readFileSync("./template.md", "utf8"), rest, { body });

    pulls_formatted.push(formatted);
  }

  /*

  I was going to include issues, but it's too much work to format after the fact. It's difficult
  to distinguish issues that were closed because they were fixed, rather than closed as duplicates
  or otherwise irrelevant to the release. We should just have good PR titles

  let closed_issues = await getClosedIssues(octokit, owner, repo, since);

  closed_issues = closed_issues.sort((a, b) => {
    return (a.closed_at < b.closed_at) ? -1 : ((a.closed_at > b.closed_at) ? 1 : 0);
  })

  let closed = [];

  for (const issue of closed_issues) {
    const issue_slug = issue.html_url.match(/issues\/\d+/)[0];
    closed.push({
      title: issue.title,
      issue_slug,
      issue_url: issue.html_url,
      author_slug: issue.user.login,
      author_url: issue.user.html_url
    })
  }

  let issues_formatted = [];
  for (const issue of closed) {
    const formatted = Mustache.render(fs.readFileSync("./template.md", "utf8"), issue);

    issues_formatted.push(formatted);
  }
  */

  let output = Mustache.render(fs.readFileSync("./output-template.md", "utf8"), {}, {
    //issues: issues_formatted.join("\n\n"),
    pulls: pulls_formatted.join("\n\n")
  });
  output = prettier.format(output, { parser: "markdown" });

  fs.writeFileSync("./output.md", output);
};

const error = (error) => {
  console.error(error);
  process.exit(1);
};

const getMergedPRs = async (octokit, owner, repo, since) => {
  // TODO: The script doesn't handle pagination. If we merge more than 100 pull requests between
  // releases then this becomes a problem. But if we wait that long, we're doing something wrong :)
  let res = await octokit.search.issuesAndPullRequests({
    q: `repo:cloudflare/wrangler is:pr base:master merged:>=${since} sort:updated-desc`,
  });

  console.log(res.url);

  if (res.status != 200) {
    error(`Failed to fetch res got ${res.status}`);
  }

  if (res.data.total_count > 100 || res.data.incomplete_results) {
    error(`You waited too long to make a wrangler release :(((( the changelog script doesn't
support pagination, so you'll have to make the changelog in two parts.

Edit the script to remove this check to proceed, or implement pagination`);
  }

  return res.data.items;
};

const getClosedIssues = async (octokit, owner, repo, since) => {
  // TODO: The script doesn't handle pagination. If we close more than 100 issues between
  // releases then this becomes a problem. But if we wait that long, we're doing something wrong :)
  let res = await octokit.search.issuesAndPullRequests({
    q: `repo:cloudflare/wrangler is:issue is:closed closed:>=${since} sort:updated-desc`,
  });

  console.log(res.url);

  if (res.status != 200) {
    error(`Failed to fetch res got ${res.status}`);
  }

  if (res.data.total_count > 100 || res.data.incomplete_results) {
    error(`You waited too long to make a wrangler release :(((( the changelog script doesn't
support pagination, so you'll have to make the changelog in two parts.

Edit the script to remove this check to proceed, or implement pagination`);
  }

  return res.data.items;
};

program
  .version("0.0.1")
  .arguments("<owner> <repo> <since>")
  .description("generate changelog", {
    owner: "repo owner",
    repo: "repo name",
    since: "prs merged since",
  })
  .action(generate)
  .parse();
