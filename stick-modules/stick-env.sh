#!/bin/sh

die () {
    die_with_status 1 $1
}

die_with_status () {
    STATUS=$1
    shift
    echo "$*"
    exit "$STATUS"
}

usage() {
    echo "$USAGE"
    exit 1
}

HELP=0
PROJECT_DIR=`pwd`
CONFIG_FILE=".stick.cfg"

# fixup the project dir if possible
while [ ! -f "$PROJECT_DIR/$CONFIG_FILE" ]; do
    PROJECT_DIR=$PROJECT_DIR/..
    if [ ! -d "$PROJECT_DIR" ]; then
        # restore pwd as project dir and break
        PROJECT_DIR=`pwd`
        break;
    fi
done

ISSUES_DIR=$PROJECT_DIR/issues
STATE_DIR=$PROJECT_DIR/state
ATTACH_DIR=$PROJECT_DIR/files
STATE_LINK_BASE=../../issues
STATES=$STATE_DIR/*
DEFAULT_STATE=open
DEFAULT_EXT=.md

TREE_CMD=$(which tree)
if [ -x $TREE_CMD ]; then
    TREE_CMD="$TREE_CMD -C"
else
    TREE_CMD="ls -l --color=always"
fi
