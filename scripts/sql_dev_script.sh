#!/bin/sh

# Helper to run a sql_dev_script in `../sql_dev_scripts`

if [ "$1" = "-h" ] || [ "$1" = "--help" ]
then
    echo "Example usage:"
    echo "    ./scripts/sql_dev_script.sh ./sql_dev_scripts/generic_test_data.sql"
    exit 0
fi

if [ ! -f "$1" ]
then
    echo "Fatal: sql script $1 does not exist"
    exit 1
fi

db_url="$(cat .env | grep DATABASE_URL | sed 's/DATABASE_URL=//g')"

psql $db_url < $1
