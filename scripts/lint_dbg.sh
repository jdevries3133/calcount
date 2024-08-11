#!/bin/sh

if [ ! -z "$(grep -rnc 'dbg!' src | grep -v ':0$')" ]
then
    echo "Fatal: found lingering dbg! statements!"
    exit 1
fi
