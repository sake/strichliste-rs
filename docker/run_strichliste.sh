#!/bin/sh

# in podman / has permissions which forbid spwaning processes as another user
chmod 0755 /

nginx

export BIND_ADDRESS=localhost:3030
export DB_FILE=/var/lib/strichliste-rs/strichliste.sqlite

chown strichliste /var/lib/strichliste-rs/

su -c strichliste-rs strichliste
