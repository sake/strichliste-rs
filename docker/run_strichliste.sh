#!/bin/sh

nginx

export BIND_ADDRESS=localhost:3030
export DB_FILE=/var/lib/strichliste-rs/strichliste.sqlite

chown strichliste /var/lib/strichliste-rs/

su -c strichliste-rs strichliste
