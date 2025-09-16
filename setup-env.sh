#!/bin/bash
rm -rf ./.embuild
ln -s /.embuild ./.embuild

chmod +x /root/export-esp.sh
. /root/export-esp.sh
