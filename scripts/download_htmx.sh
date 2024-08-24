#!/bin/sh

set -eux

HTMX_CHECKSUM="e1746d9759ec0d43c5c284452333a310bb5fd7285ebac4b2dc9bf44d72b5a887"
HTMX_VERSION="2.0.2"

if [ ! -f ./src/htmx-$HTMX_VERSION.vendor.js ]
then
    curl -L https://unpkg.com/htmx.org@$HTMX_VERSION > src/htmx-$HTMX_VERSION.vendor.js
fi

checksum="$(
    openssl dgst -sha256 -hex src/htmx-$HTMX_VERSION.vendor.js \
    | sed 's/.*= \(.*\)/\1/g'
)"

if [ "$checksum" != "$HTMX_CHECKSUM" ]; \
then
    rm ./src/htmx-$HTMX_VERSION.vendor.js
    exit 1
fi
