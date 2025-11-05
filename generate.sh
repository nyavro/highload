#!/bin/bash

rm -r ./api
docker run --rm \
  -v ${PWD}:/local openapitools/openapi-generator-cli generate \
  -i /local/openapi/openapi.json \
  -g rust-axum \
  -o /local/api
