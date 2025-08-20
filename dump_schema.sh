#!/bin/bash
# dump_schema.sh
# Usage: ./dump_schema.sh > schema.sql

# Load .env file (must contain DATABASE_URL)
set -a
source .env
set +a

if [ -z "$DATABASE_URL" ]; then
  echo "Error: DATABASE_URL is not set in .env"
  exit 1
fi

pg_dump --schema-only --dbname="$DATABASE_URL"
