#!/bin/bash

CRATE=$1

if [[ -z $CRATE ]]; then
	echo "Please provide the crate name"
	exit 1
fi

if [[$CRATE != "avalanche-types" || $CRATE != "avalanche-consensus"]]; then
	echo "Invalid crate name"
	exit 1
fi

echo "Checking $CRATE"

published_version=$(cargo search $CRATE | head -1 | cut -d'"' -f 2)
echo $published_version

current_version=$(cargo pkgid -p $CRATE | cut -d "#" -f2)
echo $avalanche_types_current_version

if [ $published_version = $current_version ]; then
	echo "The current and published versions are equal."
	exit 1
else
	echo "The current and published versions are different."
	exit 0
fi
