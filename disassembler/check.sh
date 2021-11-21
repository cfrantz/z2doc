#!/bin/bash

set -e

CA65=/usr/local/cc65/bin/ca65
LD65=/usr/local/cc65/bin/ld65

EXPECTED_SHA256="b1f7782d6099e3da158f45e9b32112c3ee361bd3c74f1fc14db88ec8eeaa1850"

cargo run -- --logging=info -d zelda2.yaml zelda2.nes >/tmp/z2.asm
${CA65} -o /tmp/z2.o /tmp/z2.asm
${LD65} -o /tmp/z2.nes -C zelda2.cfg /tmp/z2.o

SHA256=$(sha256sum /tmp/z2.nes)
echo ${SHA256}
echo "b1f7782d6099e3da158f45e9b32112c3ee361bd3c74f1fc14db88ec8eeaa1850  Expected sum"

SHA256=$(echo ${SHA256} | cut -f1 -d' ')
if [[ ${SHA256} != ${EXPECTED_SHA256} ]]; then
    echo "FAIL"
    exit 1
fi

echo "PASS"
