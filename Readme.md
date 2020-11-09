# Dev Environment

The `Dockerfile.dev` file builds a container comprising the Strichliste Frontend and the necessary config to run the backend.
The backend executable must be added at runtime via a bindmount.
That also means that the executable should be first build with `cargo build`.

The following commands build the container and execute it with the required mounts and port forwardings.
Note that podman is a docker drop in replacement which does not require root access.
The good old docker can be used alternatively.

`$ podman build -t strichliste-dev -f Dockerfile.dev .`
`$ podman run -it --rm -v ./target/debug/strichliste-rs:/usr/local/bin/strichliste-rs -v ./dev:/var/lib/strichliste-rs -p8080:8080 strichliste-dev`
