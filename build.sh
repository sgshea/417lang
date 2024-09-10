#!/bin/bash

os=$(uname -s)
kernelversion=$(uname -v)

allpassed=1
function check-for {
    for str in "$@"; do
	if [[ "$(command -v $str)" == "" ]]; then
	    printf "Not found: %s\n" "$str"
	    allpassed=0
	fi
    done
}

#
# Install dependencies for CSC 417 parser on Ubuntu 22.04:
#

# Some systems, like NCSU VCL, lack the standard set of C headers,
# which is why the libc-dev package may be needed.

if [[ $(uname -v) == *"Ubuntu"* ]]; then
    echo "On Ubuntu.  Installing dependencies to run Rust interpreter now (as root)..."
    sudo apt-get install -y \
	 cargo \
	 git
else
    echo "Not Ubuntu.  Checking for dependencies (not installing)..."
    check-for git cargo
fi

check-for gawk

if [[ ! -r ./src* ]]; then
    echo "Missing interpreter source code!"
    allpassed=0
fi

if [[ $allpassed -eq 1 ]]; then 
    echo "All dependencies installed, all code compiled."
    exit 0
else
    exit -1
fi
