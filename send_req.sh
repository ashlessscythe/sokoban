#!/bin/bash

# define url
# URL="http://localhost:8000"
URL="https://localhost:8000"

# if no args provided print usage
if [ $# -eq 0 ]; then
	echo "Usage: $0 TYPE ENDPOINT COUNT"
	exit 1
fi

# type of req
TYPE=$1

# define endpoint as second arg
ENDPOINT=$2

# count of req to send
COUNT=$3

## send req as json
for ((c = 1; c <= $COUNT; c++)); do
	# log
	# generate random 4 char string
	NAME=$(cat /dev/urandom | tr -dc 'a-zA-Z' | fold -w 4 | head -n 1)

	# use name as email @local.eml
	EMAIL=$NAME"@local.eml"

	# use name as id name_s_id
	ID=$NAME"_s_id"

	echo "Sending $TYPE request to $URL/$ENDPOINT with name: $NAME, email: $EMAIL, user_id: $ID"
	curl -X $TYPE $URL/$ENDPOINT -d '{"name": "'$NAME'", "email": "'$EMAIL'", "user_id":"'$ID'"}'
	# send random either In or Out
	INOUT=$(shuf -n 1 -e In Out)
	curl -X $TYPE $URL/punch/$ID -d '{"in_out":"'$INOUT'"}'
done
