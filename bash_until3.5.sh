#!/usr/bin/env bash
set -e

# This worked until xxd was removed in 3.5
# Convert binary to hex so sed can edit it
# then convert back to binary

xochitl='/usr/bin/xochitl'
tmpfile='/tmp/signature.xochitl.tmp'
guilty=$(printf "Sent from my" | xxd -p)
absolve=$(printf "\0ent from my" | xxd -p)

apply() {
    xxd -p $xochitl | sed "s/$guilty/$absolve/" | xxd -p -r >"$tmpfile"
    chmod +x $tmpfile && mv $tmpfile $xochitl
}

revert() {
    xxd -p $xochitl | sed "s/$absolve/$guilty/" | xxd -p -r >"$tmpfile"
    chmod +x $tmpfile && mv $tmpfile $xochitl
}
