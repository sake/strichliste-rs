#!/bin/sh

SL_BIN=/strichliste-build/strichliste-rs

# start nginx, starts in the background itself
nginx

# start strichliste and watch for open of the file (happens during compilation), then kill old proc and start new
while [ true ]
do
	$SL_BIN &
	SL_PID=$!
	inotifywait -e open $SL_BIN
	kill $SL_PID
done
