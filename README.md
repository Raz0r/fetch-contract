# fetch-contract

### Download the source code of a verified smart contract from etherscan.io by its address

Verified smart contracts on etherscan.io may have multiple files, this tool fetches all the files and stores them locally.
To access etherscan.io API you will need an [API key](https://etherscan.io/myapikey) which can be requested by any registered user.
The API key can be supplied via an environmental variable `ETHERSCAN_APIKEY` or an argument `-k`.