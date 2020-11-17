# About strichliste-rs

This project is a reimplementation of the backend of the marvellous [Strichliste](https://www.strichliste.org) created by the a few guys from Backspace Bamberg (https://www.hackerspace-bamberg.de).
Also see https://github.com/strichliste for the source of the original.

Right now this is just an elaborate Hello World program to teach myself rust, so don't expect high quality code.
There is also a docker container bundling the frontend of the original Strichliste.

During the development one thing became apparent.
Copying the functionality from the PHP entity framework created more complicated code than it is probably necessary.
However without changing the CRUD interface, the frontend expects, it does not make sense to try to fight recursive entities.

Maybe I will try something as fancy as [elm](https://elm-lang.org/) and redo the frontend as well.

As long as this is  not the case this must be considered as a toy project to learn a new language.
Use the original Strichliste if you need something actively maintained.

# Dev Environment

The `Dockerfile.dev` file builds a container comprising the Strichliste Frontend and the necessary config to run the backend.
The backend executable must be added at runtime via a bindmount.
That also means that the executable should be first build with `cargo build`.

The following commands build the container and execute it with the required mounts and port forwardings.
Note that podman is a docker drop in replacement which does not require root access.
The good old docker can be used alternatively.

```
$ podman build -t strichliste-dev -f Dockerfile.dev .
$ podman run -it --rm -v ./target/debug/:/strichliste-build/ -v ./dev:/var/lib/strichliste-rs -p8080:8080 strichliste-dev
```
