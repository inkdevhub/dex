# DEX - UniswapV2
This folder contains the line by line implementation of [uniswap-v2 core](https://github.com/Uniswap/v2-core) and [uniswap-v2 periphery](https://github.com/Uniswap/v2-periphery) with its tests.   
First version: [wasm-showcase-dapps](https://github.com/AstarNetwork/wasm-showcase-dapps)

### Purpose
This is an unaudited full dex implementation ready to be used.

### Status
- :white_check_mark: contracts
- :white_check_mark: integration tests
- :white_large_square: UI (January 2023)
- :white_large_square: Audit

### Versions
[ink! 4.0 RC](https://github.com/paritytech/ink/tree/v4.0.0-rc)   
[openbrush 3.0.0 beta.1](https://github.com/727-Ventures/openbrush-contracts/tree/3.0.0-beta.1)

### License
Apache 2.0

## 🏗️ How to use - Contracts
##### 💫 Build
Use these [instructions](https://use.ink/getting-started/setup) to set up your ink!/Rust environment    
Run this command in the contract folder:

```sh
cargo contract build
```

##### 💫 Run unit test

```sh
cargo test
```
##### 💫 Deploy
First start your local node.  
Deploy using polkadot JS. Instructions on [Astar docs](https://docs.astar.network/docs/wasm/sc-dev/polkadotjs-ui)

##### 💫 Run integration test
First start your local node. Recommended [swanky-node](https://github.com/AstarNetwork/swanky-node) v0.13.0

```sh
yarn
yarn compile
yarn test:typechain
```

##### 💫 Deployed contracts

###### Shibuya
Factory [Zuyf2wq2WXknr82Dp7Et46qmRJpWFY7bzdSM87MN5RqzYLv](https://shibuya.subscan.io/account/Zuyf2wq2WXknr82Dp7Et46qmRJpWFY7bzdSM87MN5RqzYLv)    
WNative [XaE8oMj2rYFv6SQKGPKxGDLrdZqhduUHrwJU7pDfUke46Kx](https://shibuya.subscan.io/account/XaE8oMj2rYFv6SQKGPKxGDLrdZqhduUHrwJU7pDfUke46Kx)    
Router [ZtkfGHXkfcimYf9cXJdjxytw5Pzp3ucnZM51kBms5Eiu2PH](https://shibuya.subscan.io/account/ZtkfGHXkfcimYf9cXJdjxytw5Pzp3ucnZM51kBms5Eiu2PH)   
USDC [X3aX6HYcKrm3ZZSna7PR45Doh9tKyjaiUBLBWWMRfpQB4u6](https://shibuya.subscan.io/account/X3aX6HYcKrm3ZZSna7PR45Doh9tKyjaiUBLBWWMRfpQB4u6)   
USDT [YDBTHeQC2d5EuWiKFnvxaUipd67Va7Kx7AuiQaXydExkfPG](https://shibuya.subscan.io/account/YDBTHeQC2d5EuWiKFnvxaUipd67Va7Kx7AuiQaXydExkfPG)    
USDC/SBY-LP [YeoXYLUimoyHmn79FwYknWp89yX265i2Rq8zdAf5DkCiRz8](https://shibuya.subscan.io/account/YeoXYLUimoyHmn79FwYknWp89yX265i2Rq8zdAf5DkCiRz8)    
USDT/SBY-LP [Z8q71nvirYBbxzmhcVoeiXZ1oL3b64zjxhVMxbmBvvzxiWs](https://shibuya.subscan.io/account/Z8q71nvirYBbxzmhcVoeiXZ1oL3b64zjxhVMxbmBvvzxiWs)    

To interact with contracts on Shibuya, use _Add an existing contract_ in [polkadotjs UI](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Frpc.shibuya.astar.network#/contracts) and enter contract address and the .contract file

---
## 🏗️  UI
Coming in January
