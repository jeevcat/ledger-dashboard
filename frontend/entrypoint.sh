#!/bin/sh

set -e

echo "Serializing environment:"

pwd

react-env --dest .

cat __ENV.js

exec "$@"