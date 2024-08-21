#!/bin/bash

# consult https://www.iana.org/time-zones for the latest versions,
# update CODE and DATA below.

# Versions to use.
CODE=2024a
DATA=2024a

set -e

cd $(dirname $0)

WORKDIR=$PWD

if [ ! -f tzcode$CODE.tar.gz ]; then
  curl -sS -L -O https://www.iana.org/time-zones/repository/releases/tzcode$CODE.tar.gz
fi

if [ ! -f tzdata$DATA.tar.gz ]; then
  curl -sS -L -O https://www.iana.org/time-zones/repository/releases/tzdata$DATA.tar.gz
fi

rm -rf work
mkdir -p work
cd work

tar xzf ../tzcode$CODE.tar.gz
tar xzf ../tzdata$DATA.tar.gz

make CFLAGS=-DSTD_INSPIRED AWK=awk TZDIR=${WORKDIR}/zoneinfo posix_only
