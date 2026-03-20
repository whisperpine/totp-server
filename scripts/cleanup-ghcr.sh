#!/usr/bin/env bash

# Purpose: batch delete packages in ghcr
# Usage: bash path/to/cleanup-ghcr
# Dependencies: curl, jq
# Date: 2026-03-20
# Author: Yusong

# This script is meant to be used in github actions.

set -e

# Defines an array of required environment variables.
required_vars=(
  "GITHUB_TOKEN"
  "OWNER"
  "PACKAGE_NAME"
  "ORG_OR_USER"
)
# Ensures all required environment variables aren't empty.
for var in "${required_vars[@]}"; do
  if [ -z "${!var}" ]; then
    echo "Error: Environment variable '$var' is missing or empty."
    exit 1
  fi
done

# "latest" or semantic versions (prefix "v" is allowed).
regex_to_keep='^(latest|v?[0-9]+\.[0-9]+\.[0-9]+(-.+)?)$'

# The endpoints of organization and user are different.
org_endpoint="https://api.github.com/orgs/${OWNER}/packages/container/${PACKAGE_NAME}/versions"
user_endpoint="https://api.github.com/users/${OWNER}/packages/container/${PACKAGE_NAME}/versions"

# Assigns "endpoint" according to ORG_OR_USER.
if [ "$ORG_OR_USER" = "org" ]; then
  endpoint="$org_endpoint"
elif [ "$ORG_OR_USER" = "user" ]; then
  endpoint="$user_endpoint"
else
  echo "Error: ORG_OR_USER can only be 'org' or 'user'. But received '$ORG_OR_USER'."
  exit 1
fi

get_versions() {
  curl -s -L \
    -H "Authorization: Bearer ${GITHUB_TOKEN}" \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2026-03-10" \
    "$endpoint" | jq -c '.[]'
}

delete_version() {
  id="$1"
  curl -s -L -X DELETE \
    -H "Authorization: Bearer ${GITHUB_TOKEN}" \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2026-03-10" \
    "${endpoint}/${id}"
}

# Remove all packages that don't match the regex.
get_versions | while read -r version; do
  id=$(echo "$version" | jq -r '.id')
  tags=$(echo "$version" | jq -r '.metadata.container.tags[]')

  keep=0
  for tag in $tags; do
    if [[ "$tag" =~ $regex_to_keep ]]; then
      keep=1
    fi
  done
  # Keep untagged packages (which may be the manifests referred by an image).
  if [ -z "$tags" ]; then
    keep=1
  fi

  if [[ "$keep" == 0 ]]; then
    delete_version "$id"
  fi
done
