#!/bin/bash
# Bash script to prepare environment and run Nessie on jsonfile

# Load nvm
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"

# Use Node 20
nvm use 20

# Go to Nessie folder
cd ~/nessie || exit

# Optional: reinstall dependencies if needed
npm install ~/node-jsonfile
npm install mocha nyc

# Run Nessie to generate tests (5 tests example)
cargo +nightly run -- --lib-name jsonfile --num-tests 5 --lib-src-dir ~/node-jsonfile --test-gen-mode OGNessie

# Run the generated tests
./node_modules/.bin/mocha test/metatest.js

# Optional: get coverage
./node_modules/.bin/nyc ./node_modules/.bin/mocha test/metatest.js
