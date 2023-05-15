#set shell := ["sh"]

# Run a local webserver to test the site.
serve:
    @cargo run
    @cargo install basic-http-server
    @echo Running test server at http://localhost:4000/
    # Run entr to regenerate the site whenever a post changes.
    # Set Ctrl-C to stop both the background server and the updater daemon.
    @(trap 'kill 0' SIGINT; ~/.cargo/bin/basic-http-server public/ & (find site/ | entr cargo run) )

build:
    cargo run
