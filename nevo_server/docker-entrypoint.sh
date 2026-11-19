#!/bin/sh
set -e

attempt=0
until npm run migration:run; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 30 ]; then
    echo "Migrations failed after 30 attempts" >&2
    exit 1
  fi
  sleep 2
done

exec "$@"
