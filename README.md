# Minimum Viable Product for Kairos - A native Casper Transaction Zk Rollup System
This repo contains a premature demo version of a Casper native transaction rollup system. Many design decisions of this repository have changed and it is not up to date with the [official Kairos project](https://github.com/cspr-rad/kairos). 

Kairos-lab is a private research branch by @jonas089 with @Rom3dius as a collaborator. The goal was to quickly demo the Kairos rollup system and test new state implementations, trees, proving backends, contracts and L1 target architecture. 

### Testing
In order to test, make sure you have [cargo-nextest](https://nexte.st) and [docker-compose](https://docs.docker.com/compose/install/#scenario-two-install-the-compose-plugin) installed.
You might also need the [jq](https://jqlang.github.io/jq/) cli tool. It comes preinstalled on most linux distros.
Executing `cargo nextest run` will automatically spawn a network using CCTL and a postgresql database.
The environment will stay running after test execution ends until explicitly stopped using the command `docker-compose down` or `docker compose down`. The reasoning behind this is to keep the time waiting on the images to spin up to a minimum while developing and testing the code.

### Setting up Risc0

```
cargo binstall cargo-risczero@1.0.0-rc.5
cargo risczero install
```

To verify the installation:

```
cargo risczero --version 
(should be 1.0.0-rc.5)
rustup toolchain list 
(should have 'risc0')
```