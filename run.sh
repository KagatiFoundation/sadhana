#!/bin/sh

option="${1}"
case ${option} in
    -i) 
    cargo run index
    ;;
    -s)
    cargo run
    ;;
    clean)
    set -xe
    rm -fr ./spy-db
    redis-cli flushall
    ;;
    *)
    echo "Usage: ./run.sh [-i|-s]"
    ;;
esac