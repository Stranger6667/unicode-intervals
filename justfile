# Cut a Rust release: `just release-rust 0.3.2`, then `just push-release rust 0.3.2`.
release-rust version: (_release "rust" "crates/unicode-intervals" "CHANGELOG.md" version)

# Cut a Python release: `just release-python 0.4.0`, then `just push-release python 0.4.0`.
release-python version: (_release "python" "crates/unicode-intervals-py" "crates/unicode-intervals-py/CHANGELOG.md" version)

# Push the release commit and its tag; the tag triggers the publishing workflow.
push-release kind version:
    git push origin main {{ kind }}-v{{ version }}

# Bump the crate version, date the changelog, commit and tag.
_release kind crate changelog version:
    #!/usr/bin/env bash
    set -euo pipefail
    test -z "$(git status --porcelain)" || { echo "Working tree is not clean" >&2; exit 1; }
    grep -qx '## \[Unreleased\]' {{ changelog }}
    previous=$(sed -n 's/^version = "\(.*\)"$/\1/p' {{ crate }}/Cargo.toml | head -1)
    sed -i '0,/^version = ".*"$/s//version = "{{ version }}"/' {{ crate }}/Cargo.toml
    sed -i "s/^## \[Unreleased\]$/## [Unreleased]\n\n## [{{ version }}] - $(date +%F)/" {{ changelog }}
    # Compare links, for changelogs that have them.
    sed -i "s|^\[Unreleased\]: \(.*\)/compare/.*\.\.\.HEAD$|[Unreleased]: \1/compare/{{ kind }}-v{{ version }}...HEAD\n[{{ version }}]: \1/compare/{{ kind }}-v$previous...{{ kind }}-v{{ version }}|" {{ changelog }}
    # Install snippet in the README tracks the Rust `major.minor`.
    if [ "{{ kind }}" = rust ]; then
        minor=$(echo "{{ version }}" | cut -d. -f1,2)
        sed -i "s/^unicode-intervals = \".*\"$/unicode-intervals = \"$minor\"/" README.md
    fi
    cargo check --quiet -p "$(sed -n 's/^name = "\(.*\)"$/\1/p' {{ crate }}/Cargo.toml | head -1)"
    git commit --quiet --signoff --all --message "chore({{ kind }}): Release {{ version }}"
    git tag {{ kind }}-v{{ version }}
    echo "Tagged {{ kind }}-v{{ version }}; publish with: just push-release {{ kind }} {{ version }}"
