#!/bin/sh
# Run inside the separate EC2 deployment directory after transferring the image,
# compose file, public runtime settings and securely provisioned secret files.
# Deliberately does not edit DNS, Caddy, unrelated containers, or AWS permissions.
set -eu
cd "$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
for required in .env.ec2 .secrets/openai_api_key .secrets/diagram_scan_token; do
    if [ ! -s "$required" ]; then
        echo "Missing required runtime file: $required" >&2
        exit 1
    fi
done
# Do not source .env files or print expanded secret-bearing Docker configuration.
docker compose --env-file .env.ec2 -f compose.ec2.yaml config --quiet
docker compose --env-file .env.ec2 -f compose.ec2.yaml up -d --no-build --wait --wait-timeout 120 diagram-api
docker compose --env-file .env.ec2 -f compose.ec2.yaml exec -T diagram-api node healthcheck.mjs
printf '%s\n' 'Diagram container healthy. Verify HTTPS proxy and authorization before client use.'
