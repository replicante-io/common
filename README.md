# Replicante Common Crates
A collection of crates shared by [replicante](https://replicante.io)'s workspaces.


## Usage
Add this repo as a submodule to the workspace that needs it.
Update the top workspace file with the needed crates if the workspace requires it.


## Development
Because this repo is meant to be a submodule inside of another workspace it comes
without a `Cargo.toml` file.

It does have a `Cargo.workspace.toml` which is a workspace listing all crates.
To use cargo comands from the repo root add a symlink named `Cargo.toml`:
```bash
ln -s Cargo.workspace.toml Cargo.toml
```


## Code of Conduct
Our aim is to build a thriving, healthy and diverse community.  
To help us get there we decided to adopt the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/)
for all our projects.

Any issue should be reported to [stefano-pogliani](https://github.com/stefano-pogliani)
by emailing [conduct@replicante.io](mailto:conduct@replicante.io).  
Unfortunately, as the community lucks members, we are unable to provide a second contact to report incidents to.  
We would still encourage people to report issues, even anonymously.

In addition to the Code Of Conduct below the following documents are relevant:

  * The [Reporting Guideline](https://www.replicante.io/conduct/reporting), especially if you wish to report an incident.
  * The [Enforcement Guideline](https://www.replicante.io/conduct/enforcing)
