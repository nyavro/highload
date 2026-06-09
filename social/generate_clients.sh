#!/bin/bash
rm -r ./clients

docker run --rm \
  -v ${PWD}:/local openapitools/openapi-generator-cli generate \
  -i /local/openapi/messenger/openapi.json \
  -g rust \
  -o /local/clients/messenger \
  --additional-properties=library=reqwest,packageName=messenger_client
