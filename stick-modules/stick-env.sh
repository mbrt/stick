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

exe_avail() {
    hash "$1" 2>/dev/null
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
TAGS_DIR=$PROJECT_DIR/tags
TICKET_LINK_BASE=../../issues
STATES=$STATE_DIR/*
DEFAULT_STATE=open
DEFAULT_EXT=.md

if exe_avail tree; then
    TREE_CMD="tree -C"
else
    TREE_CMD="ls -l --color=always"
fi

print_ticket_info() {
    file_path="$1"
    ticket_state="$2"
    ticket_id=$(basename "$file_path" "$DEFAULT_EXT")
    first_line=$(head -1 "$file_path")
    title=$(echo $first_line | sed "s/$ticket_id\s*[:_-]*\s*//I")
    if [ -z "$ticket_state" ]; then
        echo "$ticket_id: $title"
    else
        echo "$ticket_id [$ticket_state]: $title"
    fi
}
