#!/bin/bash

CRATE=$1
if [[ $CRATE == "avalanche-types" ]]; then
	echo "Checking avalanche-types"

	avalanche_types_published_version=$(cargo search avalanche-types | head -1 | cut -d'"' -f 2)
	echo $avalanche_types_published_version

	avalanche_types_current_version=$(cargo pkgid -p avalanche-types | cut -d "#" -f2)
	echo $avalanche_types_current_version

	if [ $avalanche_types_published_version = $avalanche_types_current_version ]; then
		echo "The current and published versions are equal."
        exit 1
	else
		echo "The current and published versions are different."
        exit 0
	fi

elif [[ $CRATE == "avalanche-consensus" ]]; then
	echo "Checking avalanche-consensus"

	avalanche_consensus_published_version=$(cargo search avalanche-consensus | head -1 | cut -d'"' -f 2)
	echo $avalanche_consensus_published_version

	avalanche_consensus_current_version=$(cargo pkgid -p avalanche-consensus | cut -d "#" -f2)
	echo $avalanche_consensus_current_version

	if [ $avalanche_consensus_published_version = $avalanche_consensus_current_version ]; then
		echo "The current and published versions are equal."
        exit 1
	else
		echo "The current and published versions are different."
        exit 0
	fi

else
	echo "Invalid crate name"
    exit 1
fi
