#!/bin/bash
set -e

. /root/export-esp.sh 

cargo build --release

source setup-idf.sh

chmod +x merged.sh

./merged.sh

# rm -rf merged.sh
