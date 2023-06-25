# Warp CLI

**_This tool is still in early stages of development. Please report all issues you've found here on GitHub_**
**_All-in-one productivity toolchain for building, testing, and deploying Archway Smart Contracts._**

The tool has been made adapted for the [EvolvNFT](https://evolvnft.com) project for the Archway Hackathon.

# Requirements

This tool was built to support the current toolstack, so everything you'd normally need for developing smart contracts is still required:

- Rust 1.50+,
- `archwayd` CLI tool
- Node & Npm if you want to run tests,
- Docker for building contracts and running the node

I am planning on reducing this list in the future, but it's not a priority right now.

# Changelog

v0.2.0 changelog can be found [here](CHANGELOG.md)

# Installation

You can build the binary from the main branch of this repo:

```
cargo install --git https://github.com/archway-warp/warp-cli.git
```

Alternatively, there's also a crates.io option

```
cargo install arch-warp-cli
```

# Usage

```
Usage: warp <COMMAND>

Commands:
  init    Initialize a new Warp project
  build   Build the current workspace
  deploy  Execute the 'Auto Deploy' script for the workspace (see Warp.toml)
  new     Scaffold a new contract
  node    Start the local validator node
  test    Run the JavaScript tests from the '/tests/' directory
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

## Initialize a new workspace

Use the `warp init <WORKSPACE_PATH>` command to create a new Cargo workspace preconfigured for use with the Warp CLI.

This command will clone the [warp-template](https://github.com/secret-warp/warp-template) repository and perform some basic setup. The workspace is set up to support the following features out of the box:

- Node.JS testing environment with `ts-mocha` and `chai` (tests/ directory - run `npm i` & `yarn` in there to get all the packages)
- Basic `Warp.toml` file that manages the project configuration (deployment scripts, testing setup, and more)
- A shared library for easily sharing `ExecuteMsg` and `QueryMsg` models of all contracts in the workspace (everything is taken care of by the Warp CLI)

## Scaffolding smart contract template

With `warp new <CONTRACT_NAME>` you can quickly add a new contract to the workspace. The command clones the contract and takes care of all the boilerplate setup for you:

- Adds the `msg.rs` module to the `shared` library for easy access by other contracts
- (CURRENTLY IN DEVELOPMENT) Adds an entry to the AutoDeploy script in `Warp.toml` to prepare your contract for deployment (of course you will most likely need to modify it to get it to work with your contract later on)

## Building the Contracts

To build your contracts you can use the `warp build` command:

```
Usage: warp build [OPTIONS]

Options:
  -o, --optimized  Build for production with 'workspace-optimizer' docker image
```

This is rather straightforward and works as advertised. In addition, some other commands can invoke this one before execution with a `-r` (rebuild) flag.

## Starting a Local Node

**_ATTENTION: THIS IS STILL NOT FUNCTIONAL IN THE ARCHWAY PORT - WORK IN PROGRESS_**

You can quickly start up a new development node using the `localsecret` image using the `warp node` command. This one is still only partially tested, but it is being used internally to allow the `test` subcommand to work.

## Testing your Smart Contracts

If you've ever worked with EVM chains and Hardhat, or Solana with Anchor Framework, you will probably find this command to be quite familiar. `warp test` allows you to run a full testing suite, and, subsequently execute integration and end-to-end tests for your contracts with Node.JS.

```
Run the JavaScript tests from the '/tests/' directory

Usage: warp test [OPTIONS]

Options:
  -r, --rebuild           Rebuild the contracts before running tests
  -s, --skip-environment  Don't start a new instance of localsecret for this testing session
```

Additionally, while I'm not great at TypeScript, I am also providing a small utility module (`tests/src/utils/localsecret.ts`) for making writing your tests as hastle-free as possible. Currently, the utility module contains the following utility functions:

- `getConstantineConnection()` - Returns a connection to the LCD API of a localsecret node
- `getGenesisWallets()` - Returns an array of pre-loaded genesis wallets available in LocalSecret (you don't need to remember or look up the mnemonics)
- `storeAndInitContract()` - A shorthand for uploading your `wasm` contract to the chain and making an instance of it. Useful for when you only need one instance of a given contract ever in your tests.
- `requestFaucetCoinsConstantine` - a quick helper function to get some test tokens on theConstantine-2 network.

## Finally, deploying your contracts to the Secret Network

This is the most complex and, truthfully, still the least polished command available in this CLI tool. It interfaces with the local `secretcli` installation and config to publish your smart contract to mainnet or testnet. It can be slow right now, but I'll be working on improving the performance and user experience of the deployment scripts over the coming weeks.

The `warp deploy` command uses the `autodeploy` script defined in the `Warp.toml` file. The deployment steps are executed in order from top to bottom, and scripts down in the queue can actually reference addresses of contracts that came before them. This is a feature I needed the most in my own project, as I don't exactly enjoy TypeScript and it is the main reason for the creation of Warp CLI.

### Example AutoDeploy Script

The following script will deploy three contracts to the network using the `deployer` account (`secretcli keys show deployer`). The first script specifies only the owner parameter, while the other two depend on the deployer account as well as each other:

```toml
[autodeploy]
account_id = 'deployer' # The account to use for deployment
make_labels_unique = true # Append the labels with pseudo-random numbers

[[autodeploy.steps]]
id = '$_acl' # Internal id for use in `init_msg` parameters of later contracts
contract = 'artifacts/acl.wasm' # Path to the compiled file
init_msg = '{ "default_role": "CALLER", "owner": "$account_id" }' # `$account_id` will be parsed into the actual secret address of the deployer wallet
label = 'Dapp: ACL'
coins = '' # Optional: Attach a deposit to the Instantiate call

[[autodeploy.steps]]
id = '$_system'
contract = 'artifacts/system_manager.wasm'
init_msg = '{ "acl": "$_acl", "owner": "$account_id" }'
label = 'Dapp: System Manager'

[[autodeploy.steps]]
id = '$_factory'
contract = 'artifacts/factory.wasm'
init_msg = '{ "acl": "$_acl", "system": "$_system", "owner": "$account_id" }'
label = 'Dapp: Factory'
```

## What about frontend?

It is not included as of right now. I am not a frontend developer, and as such, I can't hold opinions on what's comfortable to use in the frontend world. In the future there will be options available to include various frontends through the CLI.

# Roadmap

The tool works, but it certainly can't be considered "stable". So, in addition to adding some killer features, there is a lot of refactoring and bugfixing to be done. Please report any issues you find!

## Long-term goals

Please keep in mind that at this early stage plans can still change quite a lot, depending on what features are needed the most. This is more of a guideline at the moment.

- Improve the user experience - fix bugs and eliminate/decrease awkward wait times
- Implement contract migration mechanism as an optional or default behavior for `warp deploy` - priotity
- Add support for scaffolding various frontend templates
- Add support for templates in general - contractt templates for different versions of `cosmwasm`, or preconfigured CW standard contracts (`warp new main_token -t cw20-staking`?)
- Find out a way to automate schema generation for contract messages as much as possible
- Make interfacing with dockerized `localsecret` less verbose - `docker exec -it secretdev secret cli blah blah`
- Write a proper documentation
- Automate the `archwayd` node configuration to reduce block time for testing purposes
- Possibly remove the dependency on locally installed `archwayd` for a more 'portable' setup that works out of the box
