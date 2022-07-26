#!/bin/bash

# Our projects need nightly. Netlify already has `rustup` and `cargo` preinstalled.
if [ "$NETLIFY" == "true" ] 
then
    rustup toolchain install nightly
fi

# Build docs for Andy the Ant.
cd andy-the-ant
export RUSTDOCFLAGS="--html-in-header ./src/docs-header.html -Z unstable-options --extend-css src/extend.css" 
cargo doc --no-deps --release

cd ..
mkdir -p target
cp andy-the-ant/target/doc target

# Copy the Netlify _redirects file into the publish directory.
cp _redirects target/_redirects


