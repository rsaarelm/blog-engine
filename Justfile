#set shell := ["sh"]

# TODO: FILL YOUR REPO URL HERE
# repository := "git@github.com:[USERNAME]/[BLOG-REPO]"

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

build source='./site/':
    cargo run -- --source {{source}}

# Use local build to publish to gh-pages.
publish:
    #!/bin/sh
    rm -rf public/
    cargo run
    DIR=$(mktemp -d)
    cp -r public/* $DIR/
    cd $DIR/
    git init --initial-branch=master
    git add .
    git commit -m "Automated deployment to gh-pages"
    git push --force {{repository}} master:gh-pages
    cd -
    rm -rf $DIR/

update-flake:
    rm -rf .direnv/
    nix flake update
