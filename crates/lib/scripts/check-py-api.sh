#!/usr/bin/env bash
# Determines if `griffe` and `knope` agree about whether this is a breaking change.
# Exits with status 1 if `griffe` says it's a breaking change, but `knope` doesn't know that.
# Exits with status 0 if they both or only `knope` reports this is a breaking change.
#
# This uses `poetry` to run `griffe`, and must be executed from the project's root directory.

set -u

# `griffe` needs to run at the project root.
pushd "$(git rev-parse --show-toplevel)" || exit
trap popd EXIT

# The Python package name to check.
PY_PACKAGE="qcs_sdk"
# The name of the package that `knope` adds change logs for.
KNOPE_PACKAGE="qcs-sdk"

# Check if `griffe` says this is a breaking change.
poetry run -P crates/lib -- \
  griffe check \
    --search crates/python \
    --search crates/lib/python \
    "${PY_PACKAGE}"
api_break=$?

# Now check if `knope` knows this has "Breaking Changes".
#
# This just looks for a line mentioning what will get added to the `CHANGELOG.md`,
# and if it exists, looks for the line `### Breaking Changes`.
# If it finds both, it should be a breaking change for $KNOPE_PACKAGE.
# If it doesn't find the one of those lines, or finds other changes before `Breaking Changes`,
# then either there are no breaking changes, or they aren't breaking changes for $KNOPE_PACKAGE.
knope --dry-run release | awk -v KNOPE_PACKAGE="$KNOPE_PACKAGE" -f <(cat <<-'EOF'
  BEGIN { is_breaking = 0; }
  /^Would add the following to .*\/CHANGELOG.md: *$/ { is_scope = ($6 ~ KNOPE_PACKAGE); }
  /^### Breaking Changes$/ && is_scope { is_breaking = 1; }
  END { exit is_breaking; }
EOF
)
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

