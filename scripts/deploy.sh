#!/bin/bash
set -e

echo "Deploying Waremax API..."
caprover deploy -a waremax-api -f ./docker/captain-definition.api

echo "Deploying Waremax Frontend..."
caprover deploy -a waremax -f ./docker/captain-definition.frontend

echo "Done!"
