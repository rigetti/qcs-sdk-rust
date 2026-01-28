"""
This script is a lint helper for our PyO3 wrappers.

Given a starting directory, it recursively searches it for ``*.rs`` files,
and attempts to extract PyO3 annotations and exports from the source files.
Afterward, it may print some messages about potential mistakes.
Run the script with ``--help`` to see its options.
"""

import sys
import logging

from pyo3_linter import (
    default_macro_handlers,
    find_possible_mistakes,
    print_package_info,
    process_dir,
    parser,
    PackageConfig,
)

logging.basicConfig(level=logging.WARNING)
logger = logging.getLogger()

def main():
    args = parser.get_parser().parse_args()

    if args.log_level is not None:
        logger.setLevel(args.log_level)

    package_config = PackageConfig(root_module="qcs_sdk", internal_module="_qcs_sdk")
    annotated, exported = process_dir(args.base, package_config, default_macro_handlers())

    issues = find_possible_mistakes(package_config, annotated, exported)
    if args.show_mistakes:
        for issue in issues:
            print(issue.message)

    if args.show_package:
        print_package_info(annotated)

    if issues:
        print(f"\n {len(issues)} potential issue(s) discovered.", file=sys.stderr)
        if not args.show_mistakes:
            print("  (use --show-mistakes to see)", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
