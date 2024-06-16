# Default branch and target repository assume you're using github pages with a
# github account name that's the same as your current local login name.

# To override this, set REPO and BRANCH environment variables with your server
# and branch, for example:
#     REPO=git@example.com BRANCH=master just publish

repo := env_var_or_default('REPO', 'git@github.com:${USER}/${USER}.github.io/')
branch := env_var_or_default('BRANCH', 'gh-pages')

# Build sources into static website in ./public_html/
build source='./site/':
    rm -rf public_html/
    cargo run -- --source {{source}}

# Run a local webserver to test the site.
serve source='./site/': (build source)
    @echo Running test server at http://localhost:8080/
    # Run entr to regenerate the site whenever a post changes.
    # Set Ctrl-C to stop both the background server and the updater daemon.
    #
    # XXX: You must restart `just serve` if you add new posts after starting
    # it, entr will only run on the posts that are present when the server was
    # started.
    @(trap 'kill 0' SIGINT; caddy run & (find {{source}} ./src ./static/ ./templates/ | entr -s 'just build {{source}}; notify-send rebuilt') )

publish source='./site/':
    #!/bin/sh

    DIR=$(mktemp -d)
    cargo run -- --source {{source}} --output $DIR
    cd $DIR/ > /dev/null
    git init --initial-branch={{branch}}
    git add .
    git commit -m "Generated static site"

    read -p "About to overwrite {{branch}} at {{repo}} with built site, proceed? [y/n] " -n 1
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git push --force {{repo}} {{branch}}
    else
        echo "Aborted."
    fi

    cd - > /dev/null
    rm -rf $DIR/
