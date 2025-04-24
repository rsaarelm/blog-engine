# Blog engine

An opinionated static site generator using
[IDM](https://github.com/rsaarelm/idm) for structured site data,
[Markdown](https://daringfireball.net/projects/markdown/) for text syntax and
[Askama templates](https://github.com/askama-rs/askama) for generating HTML.
Currently geared for a personal blog plus a link collection page, you probably
want to fork this and edit the templates if you want something different.

Operate using the [Justfile](https://github.com/casey/just):

Run a local server (requires `caddy` web server, `entr` file monitor and
`notify-send` to be installed) for site content under a separate directory:

    just serve ~/work/website

Deployment assumes you're pushing to GitHub Pages, using branch `gh-pages` and
that your GitHub account name is the same as your login name.

    just publish ~/work/website

If you want to publish to a different branch and repository, use environment variables:

    REPO=/tmp/my-git BRANCH=master just publish ~/work/website
