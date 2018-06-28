Releasing common crates
=======================
Crates in the common repo are often not released directly but are used as submodules.
Beside a few exceptions that are published (for the base agent crate to be published)
there are still some steps to perform here when agents or core are released:

- [ ] Ensure tests and CI checks pass
- [ ] Bump the version number of all crates that need it
- [ ] Update changelogs with version and date
- [ ] Git commit release
- [ ] Update submodule in the agents and core repos
