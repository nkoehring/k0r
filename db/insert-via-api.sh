#!/bin/sh

key="$1"
input="$2"

test -z "$key" && echo "Usage: $0 <api-key> ./path/to/url.file" && exit 1
test ! -r "$input" && echo "Usage: $0 <api-key> ./path/to/url.file" && exit 1

xargs -a $input -I % -P4 curl -X POST localhost:8080 -H 'Content-Type: application/json' -d '{"url":"%","title":"an example","description":"totally examplary url it is","key":"'$key'"}'
