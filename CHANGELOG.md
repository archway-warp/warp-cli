# v0.3.0:

- Added an abstraction layer for commands to implement different behavior for different chains
- Abstracted Archway-specific functionalities away from the Warp core
- Added XION module to Warp
- Added `frontend` command for scaffolding a frontend quickstart for the project
- Fixed error handling and display
- Fixed gas-related issues on Archway
- Optimized the commands to work seamlessly across multiple chains (for example: starting to build on Archway, then deploying to XION)

# v0.2.2:

- Added two commands for execution and querying contract information

# v0.2.0:

- Added Constantine-3 support
- Added gas estimation for Archway
- Added contract migration support
- Added `Deployment.toml` where all contract addresses are stored premanently
- Added a network change command to the project for easy switching between testnets/mainnet --- `warp config set --network <NETWORK>`
- Improved user experience of the CLI tool
- Improved `warp deploy` command: new `-r` flag that lets the user rebuild and deploy contracts all in a single command
- Fixed errors with file permissions when making an optimized build
