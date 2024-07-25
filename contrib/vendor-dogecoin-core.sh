#!/usr/bin/env bash
set -e

function usage() {
    echo
    echo "Usage: script OPTIONS [dogecoin-core-version]"
    echo
    echo "OPTIONS:"
    echo
    echo " -f    Vendor even if there are local changes to the rust-dogecoinconsensus git index"
    echo " -h    Print this help an exit"
    echo
    echo "Example:"
    echo
    echo "    vendor-dogecoin-core v0.21.2"
    echo

    exit 0
}

if (($# < 1)) || [ "$1" == '-h' ]; then
   usage
fi

# Set default variables

if [ -z "$CORE_VENDOR_GIT_ROOT" ]; then
    CORE_VENDOR_GIT_ROOT="$(git rev-parse --show-toplevel)"
else
    CORE_VENDOR_GIT_ROOT="$(realpath "$CORE_VENDOR_GIT_ROOT")"
fi

DEFAULT_DEPEND_DIR="depend"
DEFAULT_CORE_REPO=https://github.com/dogecoin/dogecoin.git

: "${CORE_VENDOR_DEPEND_DIR:=$DEFAULT_DEPEND_DIR}"
: "${CORE_VENDOR_REPO:=$DEFAULT_CORE_REPO}"

# CP_NOT_CLONE lets us just copy a directory rather than git cloning.
# This is usually a bad idea, since it will bring in build artifacts or any other
# junk from the source directory, but may be useful during development or CI.
: "${CORE_VENDOR_CP_NOT_CLONE:=no}"

echo "Using depend directory $CORE_VENDOR_DEPEND_DIR. Set CORE_VENDOR_DEPEND_DIR to override."
echo "Using dogecoin repository $CORE_VENDOR_REPO. Set CORE_VENDOR_REPO to override."

# Parse command-line options
CORE_REV=""
FORCE=no
while (( "$#" )); do
    case "$1" in
    -h)
        echo ""
        usage
        ;;
    -f)
        FORCE=yes
        ;;
    *)
        if [ -z "$CORE_REV" ]; then
            CORE_REV="$1"
        else
            echo "WARNING: ignoring unknown command-line argument $1"
        fi
        ;;
    esac
    shift
done

echo
if [ "$CORE_REV" ]; then
    echo "Vendoring Dogecoin Core version: $CORE_REV"
else
    echo "WARNING: No Dogecoin Core revision specified. Will use whatever we find at the git repo."
fi
echo

# Check if we will do anything destructive.

if [ "$FORCE" == "no" ]; then
    if ! git diff --quiet -- "*.rs"; then
        echo "ERROR: There appear to be modified source files. Check these in or pass -f (some source files will be modified to have symbols renamed)."
        exit 2
    fi
    if ! git diff --quiet -- "$CORE_VENDOR_DEPEND_DIR"; then
        echo "ERROR: The depend directory appears to be modified. Check it in or pass -f (this directory will be deleted)."
        exit 2
    fi
fi

DIR=./dogecoin

pushd "$CORE_VENDOR_DEPEND_DIR" > /dev/null
rm -rf "$DIR" || true

# Clone the repo. As a special case, if the repo is a local path and we have
# not specified a revision, just copy the directory rather than using 'git clone'.
# This lets us use non-git repos or dirty source trees as secp sources.
if [ "$CORE_VENDOR_CP_NOT_CLONE" == "yes" ]; then
    cp -r "$CORE_VENDOR_REPO" "$DIR"
    chmod -R +w "$DIR" # cp preserves write perms, which if missing will cause patch to fail
else
    git clone "$CORE_VENDOR_REPO" "$DIR"
fi

# Check out specified revision
pushd "$DIR" > /dev/null
if [ -n "$CORE_REV" ]; then
    git checkout "$CORE_REV"
fi
SOURCE_REV=$(git rev-parse HEAD || echo "[unknown revision from $CORE_VENDOR_REPO]")
rm -rf .git/ || true
popd

# Record revision
echo "# This file was automatically created by $(basename "$0")" > ./dogecoin-HEAD-revision.txt
echo "$SOURCE_REV" >> ./dogecoin-HEAD-revision.txt

popd > /dev/null
