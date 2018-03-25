Replicante Common Crates
========================
A collection of crates shared by [replicante](https://replicante.io)'s workspaces.


Usage
-----
Add this repo as a submodule to the workspace that needs it.
Update the top workspace file with the needed crates if the workspace requires it.


Development
-----------
Because this repo is meant to be a submodule inside of another workspace it comes
without a `Cargo.toml` file.

It does have a `Cargo.workspace.toml` which is a workspace listing all crates.
To use cargo comands from the repo root add a symlink named `Cargo.toml`:
```bash
ln -s Cargo.workspace.toml Cargo.toml
```
