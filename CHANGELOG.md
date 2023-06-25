# v0.2.0:

- Added Constantine-3 support
- Added gas estimation for Archway
- Added contract migration support
- Added `Deployment.toml` where all contract addresses are stored premanently
- Added a network change command to the project for easy switching between testnets/mainnet --- `warp config set --network <NETWORK>`
- Improved user experience of the CLI tool
- Improved `warp deploy` command: new `-r` flag that lets the user rebuild and deploy contracts all in a single command
- Fixed errors with file permissions when making an optimized build
