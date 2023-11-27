#!/bin/bash
# write output of dump1090 beast to file
# first argument is ip
# second argument is port
if [ -z ${1} ] || [ -z ${2} ]; then
  echo "must specify dump1090 ip and port"
  exit 1
fi
nc -vv $1 $2 2>&1 > beast-capture-$(date +%Y%m%d).bin
