#!/bin/sh

~/.cargo/bin/bindgen \
    --no-derive-debug \
    --whitelist-type "LV2.*" \
    --whitelist-var "LV2_.*" \
    --generate-inline-functions \
    src/wrapper.h -o src/bindings.rs
