#!/bin/bash

# This script helps to deploy rust doc to nginx for preview.

cd `dirname ${BASH_SOURCE[0]}`

repo_tag=nginx:1.23.4-alpine3.17-slim

path=$PWD/target/doc
if [ ! -d $path ]; then
  echo "doc isn't ready :("
  exit 1
fi

docker rm -f doc-preview

docker run -it --name doc-preview -p 9090:80 --rm -v $path:/usr/share/nginx/html $repo_tag
