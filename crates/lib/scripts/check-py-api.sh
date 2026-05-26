#!/usr/bin/env bash
# Determines if `griffe` and `knope` agree about whether this is a breaking change.
# Exits with status 1 if `griffe` says it's a breaking change, but `knope` doesn't know that.
# Exits with status 0 if they both or only `knope` reports this is a breaking change.
#
# This uses `poetry` to run `griffe`, and must be executed from the project's root directory.

set -u

# `griffe` and `knope` need to run at the project root.
pushd "$(git rev-parse --show-toplevel)" || exit
# Tempdir is used for any "scratch" files
tmpdir=$(mktemp -d)
cleanup() {
  popd
  rm -rf "$tmpdir"
  # The `griffe` check command creates a branch (and worktree), but does not clean them up.
  git worktree list --porcelain \
    | awk '/^worktree /{wt=$2} /^branch refs\/heads\/griffe-lib-v/{print wt}' \
    | xargs -r -I{} git worktree remove --force {}
  git show-branch --list 'griffe-lib-v*' 2>/dev/null | awk -F'[][]' '{print $2}' | xargs -r git branch -D
}
trap cleanup EXIT

# The Python package name to check.
PY_PACKAGE="qcs_sdk"
# The name of the CHANGELOG that `knope` adds change logs for.
KNOPE_PACKAGE="crates/lib"

# Check if `griffe` says this is a breaking change.
poetry run -P crates/lib -- \
  griffe check \
    --search crates/python \
    --search crates/lib/python \
    "${PY_PACKAGE}"
api_break=$?

# Now check if `knope` knows this has "Breaking Changes". (Invert the "success" exit code for a match.)
knope --dry-run --verbose release > "$tmpdir/knope-dry-run.txt" 2>&1
if [[ $? -ne 0 ]] ; then
  echo "FATAL: knope failed to run" >&2
  exit 1
fi
! grep -q -E '^\s+implies rule MAJOR' "$tmpdir/knope-dry-run.txt"
marked_break=$?

if [[ $api_break == $marked_break ]]; then
  echo "griffe and knope agree about breaking changes"
  exit 0
elif [[ $api_break == 0 ]] ; then
  # This isn't an error, but it might be a surprise.
  echo "knope knows about breaking changes, but griffe doesn't report breaking changes for $PY_PACKAGE"
  exit 0
else
  echo "griffe says this is a breaking change for the $PY_PACKAGE API, but knope does not know that!"
  exit 1
fi

