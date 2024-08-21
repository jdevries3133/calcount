#!/bin/sh

if [ ! -z "$(grep -rnic 'TODO' src | grep -v ':0$')" ]
then
    echo "Fatal: just _do it_"
    exit 1
fi

