#!/bin/bash
set -e

source setup-idf.sh

cargo build --release

chmod +x merged.sh

./merged.sh

rm -rf merged.sh
