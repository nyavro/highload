#!/bin/bash

docker run --rm \
  -v ${PWD}:/local openapitools/openapi-generator-cli author template \
  -g rust-axum \
  -o /local/custom-templates
