##
## CHEAT SHEET
##
## These commands are invoked from a shell with `just [command] [args]`. Most of
## these commands quality-control the code in this workspace in some ways. Below
## is a summary of the most useful commands, ordered from the most comprehensive
## (and slowest) to the least comprehensive (and fastest).
##
## Just reference: https://just.systems/man/en/introduction.html
##
## $ just control
## Runs absolutely all checks. Good to run before committing. **Slow** because
## it includes the doctests (which are incredibly slow by themselves) across all
## relevant feature permutations (which multiplies the slowness).
##
## $ just dt [package]
##
## Runs only the doctests across all relevant feature permutations, but narrows
## it down to only one package. This is useful when writing doctests.
##
## $ just cbt
## (CBT = check, build, test.) Excludes system tests and doctests, making this a
## reasonably fast alternative that still checks a lot.
##
## $ just cbt [package]
## Narrows it down to only one package. Keep in mind that changes in one package
## can very well break tests in another.
##
## $ just nt [package]
## Runs just tests (unit + integration) for the given package.
##
## $ just cbt_probe
## Narrows down feature permutations to only one, “most likely” permutation for
## every package. Good when features are not involved in the ongoing change.
##
## $ just cbt_probe [package]
## Narrows it down to only one package.
##
## $ just self_update
## Updates Rust & Cargo, as well as installed Cargo commands.
##

##
## Public
##

# Run full pre-commit control routine against all feature permutations
control package='': _clean_rabbitmq _boot_rabbitmq (cbt package) _test_rabbitmq

# Check, build, and test
cbt package='': (_check package) (_build package) (_doctest package) (_nextest package)

# Check, build, and test a particularly testable feature permutation
cbt_probe package='': (_check_probe package) (_build_probe package) (_doctest_probe package) (_nextest_probe package)

# Run doc tests against all feature permutations only for the given package
dt package='': (_doctest package)

# Run unit and integration tests against all feature permutations
nt package='': (_nextest package)

# Upgrade all dependencies to latest versions within their individual constrains
up:
    cargo update

# View the public API surface
api package:
    cargo public-api --package '{{package}}'

##
## Internal
##

# Check all feature permutations
_check package='':
    if [ '{{package}}' = '' ]; \
    then cargo fc --fail-fast --pedantic check; \
    else cargo fc --fail-fast --pedantic --package '{{package}}' check; \
    fi
    if [ '{{package}}' = '' ]; \
    then cargo fc --fail-fast --pedantic check --release; \
    else cargo fc --fail-fast --pedantic --package '{{package}}' check --release; \
    fi

# Check a particularly testable feature permutation
_check_probe package='':
    if [ '{{package}}' = '' ]; \
    then cargo check --workspace --no-default-features --features _probe; \
    else cargo check --package '{{package}}' --no-default-features --features _probe; \
    fi
    if [ '{{package}}' = '' ]; \
    then cargo check --workspace --no-default-features --features _probe --release; \
    else cargo check --package '{{package}}' --no-default-features --features _probe --release; \
    fi

# Build all feature permutations
_build package='':
    if [ '{{package}}' = '' ]; \
    then cargo fc --fail-fast --pedantic build; \
    else cargo fc --fail-fast --pedantic --package '{{package}}' build; \
    fi
    if [ '{{package}}' = '' ]; \
    then cargo fc --fail-fast --pedantic build --release; \
    else cargo fc --fail-fast --pedantic --package '{{package}}' build --release; \
    fi

# Build a particularly testable feature permutation
_build_probe package='':
    if [ '{{package}}' = '' ]; \
    then cargo build --workspace --no-default-features --features _probe; \
    else cargo build --package '{{package}}' --no-default-features --features _probe; \
    fi
    if [ '{{package}}' = '' ]; \
    then cargo build --workspace --no-default-features --features _probe --release; \
    else cargo build --package '{{package}}' --no-default-features --features _probe --release; \
    fi

# Run doc tests against all feature permutations
_doctest package='':
    if [ '{{package}}' = '' ]; \
    then cargo fc --fail-fast --pedantic --only-packages-with-lib-target test --doc; \
    else cargo fc --fail-fast --pedantic --only-packages-with-lib-target --package '{{package}}' test --doc; \
    fi

# Run doc tests against a particularly testable feature permutation
_doctest_probe package='':
    if [ '{{package}}' = '' ]; \
    then cargo test --workspace --no-default-features --features _probe --doc; \
    else cargo test --package '{{package}}' --no-default-features --features _probe --doc; \
    fi

# Run unit and integration tests against all feature permutations
_nextest package='':
    if [ '{{package}}' = '' ]; \
    then cargo fc --fail-fast --pedantic nextest run --no-tests=pass; \
    else cargo fc --fail-fast --pedantic --package '{{package}}' nextest run --no-tests=pass; \
    fi

# Run unit and integration tests against a particularly testable feature permutation
_nextest_probe package='':
    if [ '{{package}}' = '' ]; \
    then cargo nextest run --workspace --no-default-features --features _probe --no-tests=pass; \
    else cargo nextest run --package '{{package}}' --no-default-features --features _probe --no-tests=pass; \
    fi

##
## System tests: RabbitMQ
##

# Run system tests in `test-strut-rabbitmq` (with clean-up afterward)
trmq: _clean_rabbitmq _boot_rabbitmq _test_rabbitmq

# Run system tests in `test-strut-rabbitmq` (from scratch, no clean-up)
srmq: _clean_rabbitmq _dirty_test_rabbitmq

# Run system tests in `test-strut-rabbitmq` (no clean-up)
drmq: _dirty_test_rabbitmq

# Bring down containers & volumes for `test-strut-rabbitmq`
_clean_rabbitmq:
    docker compose -f test_strut_rabbitmq/docker-compose.yml down --volumes

# Bring up containers & volumes for `test-strut-rabbitmq`
_boot_rabbitmq:
    docker compose -f test_strut_rabbitmq/docker-compose.yml up -d

# Run system tests in `test-strut-rabbitmq` (with clean-up afterward)
_test_rabbitmq:
    #!/usr/bin/env bash
    set -e
    trap 'docker compose -f test_strut_rabbitmq/docker-compose.yml down --volumes' EXIT
    set -x
    RABBITMQ_PORT=3372 cargo nextest run -p test-strut-rabbitmq --profile=rabbitmq --all-targets --run-ignored=all
    RABBITMQ_PORT=4072 cargo nextest run -p test-strut-rabbitmq --profile=rabbitmq --all-targets --run-ignored=all
    RABBITMQ_PORT=4172 cargo nextest run -p test-strut-rabbitmq --profile=rabbitmq --all-targets --run-ignored=all
    set +x

_dirty_test_rabbitmq:
    docker compose -f test_strut_rabbitmq/docker-compose.yml up -d
    RABBITMQ_PORT=3372 cargo nextest run -p test-strut-rabbitmq --profile=rabbitmq --all-targets --run-ignored=all
    RABBITMQ_PORT=4072 cargo nextest run -p test-strut-rabbitmq --profile=rabbitmq --all-targets --run-ignored=all
    RABBITMQ_PORT=4172 cargo nextest run -p test-strut-rabbitmq --profile=rabbitmq --all-targets --run-ignored=all

##
## CI
## CI-specific versions of some of the same recipes.
##

# Run full pre-publish control routine
_ci_control: _clean_rabbitmq _boot_rabbitmq _ci_cbt _ci_test_rabbitmq

# Check, build, and test
_ci_cbt: _check _build _ci_test

# Run doc, unit, and integration tests against all feature permutations
_ci_test: _doctest _ci_nextest

# Run unit and integration tests against all feature permutations
_ci_nextest:
    cargo fc --fail-fast --pedantic nextest run --profile=ci --no-tests=pass

# Run system tests in `test-strut-rabbitmq` (with clean-up afterward)
_ci_test_rabbitmq:
    #!/usr/bin/env bash
    set -e
    trap 'docker compose -f test_strut_rabbitmq/docker-compose.yml down --volumes' EXIT
    set -x
    RABBITMQ_PORT=3372 cargo nextest run -p test-strut-rabbitmq --profile=ci-rabbitmq --all-targets --run-ignored=all
    RABBITMQ_PORT=4072 cargo nextest run -p test-strut-rabbitmq --profile=ci-rabbitmq --all-targets --run-ignored=all
    RABBITMQ_PORT=4172 cargo nextest run -p test-strut-rabbitmq --profile=ci-rabbitmq --all-targets --run-ignored=all
    set +x

##
## Release & publish
## Commands for generating changelogs, tagging commits, and publishing packages to Crates.io
##

tag version package_suffix='':
    if [ '{{package_suffix}}' = '' ]; \
    then just _tag_root '{{version}}'; \
    else just _tag_sub '{{version}}' '{{package_suffix}}'; \
    fi

tag_all version:
    #!/usr/bin/env bash
    set -euxo pipefail
    just _tag_root '{{version}}'
    for d in strut_*/; do
        if [ -d "$d" ]; then
            package_suffix="${d#strut_}"
            package_suffix="${package_suffix%/}"
            just _tag_sub '{{version}}' "$package_suffix"
        fi
    done

_tag_root version:
    [ -d 'strut' ]
    git tag -a 'strut-{{version}}' -m 'chore: prepare strut v{{version}}'

_tag_sub version package_suffix:
    [ -d 'strut_{{package_suffix}}' ]
    git tag -a 'strut-{{package_suffix}}-{{version}}' -m 'chore: prepare strut-{{package_suffix}} v{{version}}'

cl_new version package_suffix='':
    if [ '{{package_suffix}}' = '' ]; \
    then just _cl_new_root '{{version}}'; \
    else just _cl_new_sub '{{version}}' '{{package_suffix}}'; \
    fi

cl_new_all version:
    #!/usr/bin/env bash
    set -euxo pipefail
    just _cl_new_root '{{version}}'
    for d in strut_*/; do
        if [ -d "$d" ]; then
            package_suffix="${d#strut_}"
            package_suffix="${package_suffix%/}"
            just _cl_new_sub '{{version}}' "$package_suffix"
        fi
    done

_cl_new_root version:
    [ -d 'strut' ]
    git cliff -c '../cliff.toml' -w 'strut' --include-path 'strut/**/*' --tag-pattern 'strut-\d+\.\d+\.\d+' --tag 'strut-{{version}}' --output 'strut/CHANGELOG.md' --unreleased

_cl_new_sub version package_suffix:
    [ -d 'strut_{{package_suffix}}' ]
    git cliff -c '../cliff.toml' -w 'strut_{{package_suffix}}' --include-path 'strut_{{package_suffix}}/**/*' --tag-pattern 'strut-{{package_suffix}}-\d+\.\d+\.\d+' --tag 'strut-{{package_suffix}}-{{version}}' --output 'strut_{{package_suffix}}/CHANGELOG.md' --unreleased

cl_prepend version package_suffix='':
    if [ '{{package_suffix}}' = '' ]; \
    then just _cl_prepend_root '{{version}}'; \
    else just _cl_prepend_sub '{{version}}' '{{package_suffix}}'; \
    fi

cl_prepend_all version:
    #!/usr/bin/env bash
    set -euxo pipefail
    just _cl_prepend_root '{{version}}'
    for d in strut_*/; do
        if [ -d "$d" ]; then
            package_suffix="${d#strut_}"
            package_suffix="${package_suffix%/}"
            just _cl_prepend_sub '{{version}}' "$package_suffix"
        fi
    done

_cl_prepend_root version:
    [ -d 'strut' ]
    git cliff -c '../cliff.toml' -w 'strut' --include-path 'strut/**/*' --tag-pattern 'strut-\d+\.\d+\.\d+' --tag 'strut-{{version}}' --prepend 'CHANGELOG.md' --unreleased

_cl_prepend_sub version package_suffix:
    [ -d 'strut_{{package_suffix}}' ]
    git cliff -c '../cliff.toml' -w 'strut_{{package_suffix}}' --include-path 'strut_{{package_suffix}}/**/*' --tag-pattern 'strut-{{package_suffix}}-\d+\.\d+\.\d+' --tag 'strut-{{package_suffix}}-{{version}}' --prepend 'CHANGELOG.md' --unreleased

##
## Bootstrap
## One-time commands for getting started with this project
##

# Check whether Rust & Cargo are on the $PATH
self_check:
    @rustc --version || { echo "rustc is not found on the $PATH"; exit 1; }
    @rustup --version || { echo "rustup is not found on the $PATH"; exit 1; }
    @cargo --version || { echo "cargo is not found on the $PATH"; exit 1; }

# Install necessary cargo commands (not Rust & Cargo themselves!)
self_bootstrap:
    cargo install cargo-update
    cargo install cargo-binstall
    cargo binstall cargo-nextest --secure
    cargo install cargo-feature-combinations
    cargo +stable install cargo-public-api --locked

# Update Rust & Cargo
self_update:
    rustup update
    cargo install-update -a
