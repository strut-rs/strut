# Contributing to Strut

🎉 Thank you in advance for helping out with this project.

Strut is open to any kind of meaningful improvement and extension.
This document provides pointers to where to start.

[issue]: https://github.com/strut-rs/strut/issues
[discord]: https://discord.gg/KNkJuMkY
[just]: https://github.com/casey/just
[github-flow]: https://docs.github.com/en/get-started/using-github/github-flow
[squash-replay]: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/about-pull-request-merges#merge-message-for-a-squash-merge
[conventional-commits]: https://www.conventionalcommits.org

## Issues

The GitHub [issue tracker][issue] is the main channel for feedback and discussions.
Any assistance in reporting, triaging, and resolving issues is welcome.

You are also welcome to join the dedicated [Discord server][discord] to chat directly with the maintainer and other users.

## Developing locally

Strut uses [just][just] for project-specific commands.
Refer to `justfile` in the root of the repository for useful commands.

The short story of it: make sure to run `just control` before committing.

## GitHub Flow

This project follows the [GitHub flow][github-flow] branching model.

The repository has one long-lived, default branch — `main`.
A commit in `main` may be tagged to signify that one or multiple Rust packages are released from that state.
Only [squash-replay commits][squash-replay] (also known as squash-merge) are allowed in `main` to keep the commit history clean.
The commit message _after_ squashing must adhere to the [Conventional Commits][conventional-commits] specification.

Additional, short-lived branches may be split from `main` and eventually squash-replayed onto it.
Only the name patterns listed below are allowed.
There is no real difference between these patterns: they are diversified only to help wrangling multiple branches during development.
Keep in mind that branches are not part of Git history and should be removed once merged.

| Branch Type | Purpose                                      |
|:------------|:---------------------------------------------|
| `feature/*` | For developing changes of any kind.          |
| `release/*` | For preparing a new release for publication. |
| `hotfix/*`  | For addressing critical bugs.                |
| `fix/*`     | For addressing non-critical bugs.            |
