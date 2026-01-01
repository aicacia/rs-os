#!/bin/sh

set -e

ENV_FILE_JS="$WORKDIR/_app/env.js"
echo -n "export const env={" > $ENV_FILE_JS
env | grep ^PUBLIC_ | while read -r line; do
  key=${line%%=*}
  value=${line#*=}
  entry="\"${key}\":\"${value}\""
  if [ -z "$first" ]; then
    first=true
    echo -n "$entry" >> $ENV_FILE_JS
  else
    echo -n ",$entry" >> $ENV_FILE_JS
  fi
done
echo -n "};" >> $ENV_FILE_JS
echo "$ENV_FILE_JS"

if [ "$#" -gt 0 ]; then
  exec "$@"
fi
