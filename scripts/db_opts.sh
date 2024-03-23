#!/bin/bash

# Usage: ./db_opts.sh conn_url {backup,restore} [tablename] filename

# Extracting database connection details from the URL
# Expected format: postgres://username:password@hostname:port/dbname

CONN_URL=$1
ACTION=$2
TABLENAME=$3 # This is now the third argument, the table name
FILENAME=$4  # The filename is now the fourth argument

# warn if fewer than 4 args
if [ $# -lt 4 ]; then
	echo "Usage: $0 conn_url {backup,restore} [tablename] filename"
	exit 1
fi

# Use regex to parse the PostgreSQL connection URL
if [[ $CONN_URL =~ postgres://([^:]+):([^@]+)@([^:]+):([^/]+)/(.+) ]]; then
	USERNAME="${BASH_REMATCH[1]}"
	PASSWORD="${BASH_REMATCH[2]}"
	HOSTNAME="${BASH_REMATCH[3]}"
	PORT="${BASH_REMATCH[4]}"
	DBNAME="${BASH_REMATCH[5]}"
else
	echo "Invalid connection URL format."
	exit 1
fi

# Export the PGPASSWORD so you don't need to enter it manually
export PGPASSWORD=$PASSWORD

case $ACTION in
backup)
	if [ -z "$TABLENAME" ]; then
		echo "No table name provided for backup."
		exit 1
	fi
	echo "Starting backup of $TABLENAME from $DBNAME to $FILENAME"
	pg_dump --data-only -h $HOSTNAME -p $PORT -U $USERNAME --table=$TABLENAME -F c -b -v -f "$FILENAME" $DBNAME
	;;
restore)
	echo "Starting restore of $DBNAME from $FILENAME"
	pg_restore -h $HOSTNAME -p $PORT -U $USERNAME -d $DBNAME -c -v "$FILENAME"
	;;
*)
	echo "Invalid action: $ACTION"
	echo "Usage: $0 conn_url {backup,restore} [tablename] filename"
	exit 1
	;;
esac

# Clear the PGPASSWORD
unset PGPASSWORD
