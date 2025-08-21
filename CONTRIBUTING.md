# Contributing to Strut

🎉 Thank you in advance for helping out with this project.

Strut is open to any kind of meaningful improvement and extension.
This document provides pointers to where to start.

[issue]: https://github.com/strut-rs/strut/issues
[discord]: https://discord.gg/KNkJuMkY
[just]: https://github.com/casey/just
[git-flow]: https://nvie.com/posts/a-successful-git-branching-model
[conventional-commits]: https://www.conventionalcommits.org

## Issues

The GitHub [issue tracker][issue] is the main channel for feedback and discussions.
Any assistance in reporting, triaging, and resolving issues is welcome.

You are also welcome to join the dedicated Strut [Discord server][discord] to chat directly with the maintainer and other users.

## Developing locally

Strut uses [just][just] for project-specific commands.
Refer to `justfile` in the root of the repository for useful commands.

The short story of it: make sure to run `just control` before committing.

## Git Flow

This project follows the “Git Flow” branching model.
If you are not familiar with it, you can read more about it in Vincent Driessen’s article, "[A successful Git branching model][git-flow]".

The repository has two long-lived branches:

- `main`: This branch contains the latest released version. Every commit on `main` is a new version and is tagged.
- `dev`: This is the primary development branch. It contains the latest stable changes that are ready for the next release.

In addition to these, we use several types of temporary, supporting branches:

| Branch Type | Branches From | Purpose                                                                                                | Merges Into      |
|:------------|:--------------|:-------------------------------------------------------------------------------------------------------|:-----------------|
| `feature/*` | `dev`         | For developing changes of any kind that will be included in an upcoming release.                       | `dev`            |
| `release/*` | `dev`         | For preparing a new release for publication. This branch allows for last-minute changes and bug fixes. | `main` and `dev` |
| `hotfix/*`  | `main`        | For addressing critical bugs in a published version that require an immediate fix.                     | `main` and `dev` |

### Merge Strategy

- **Feature Branches (`feature/*`)**: When a change is complete, its branch is squashed and rebased onto the latest `dev`. The commit message _after_ squashing must adhere to the [Conventional Commits][conventional-commits] specification.
- **Release and Hotfix Branches (`release/*`, `hotfix/*`)**: These branches are merged into both `main` and `dev` using a non-fast-forward merge (`--no-ff`) to preserve the branch history.
