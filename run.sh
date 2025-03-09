#!/bin/sh

option="${1}"
case ${option} in
    -i) 
    cargo run index
    ;;
    -s)
    cargo run
    ;;
    *)
    echo "Usage: ./run.sh [-i|-s]"
    ;;
esac