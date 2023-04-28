# Run a local webserver to test the site.
serve:
    # Generate pages
    @cargo run
    @cargo install basic-http-server
    @echo Running test server at http://localhost:4000/
    ~/.cargo/bin/basic-http-server public/

build:
    cargo run
