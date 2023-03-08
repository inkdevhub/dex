import fs from 'fs';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import type { Hash } from '@polkadot/types/interfaces/runtime';
import { Abi } from '@polkadot/api-contract';
import Token_factory from '../types/constructors/psp22_token';
import Factory_factory from '../types/constructors/factory_contract';
import Wnative_factory from '../types/constructors/wnative_contract';
import Router_factory from '../types/constructors/router_contract';
import Token from '../types/contracts/psp22_token';
import Factory from '../types/contracts/factory_contract';
import Wnative from '../types/contracts/wnative_contract';
import Router from '../types/contracts/router_contract';
import 'dotenv/config';
import '@polkadot/api-augment';

// Create a new instance of contract
const wsProvider = new WsProvider('wss://rpc.shibuya.astar.network');
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

async function main(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const deployer = keyring.addFromUri(process.env.PRIVATE_KEY);
  const tokenFactory = new Token_factory(api, deployer);
  const totalSupply = parseUnits(1_000_000).toString();
  const stableTotalSupply = parseUnits(1_000_000, 6).toString();
  const tokenContractRaw = JSON.parse(
    fs.readFileSync(__dirname + `/../artifacts/psp22_token.contract`, 'utf8'),
  );
  const tokenAbi = new Abi(tokenContractRaw);
  let { gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: tokenAbi.info.source.wasm },
    tokenAbi.constructors[0].toU8a([totalSupply, 'Apollo Token', 'APLO', 18]),
    '',
  );
  const { address: aploAddress } = await tokenFactory.new(
    totalSupply,
    'Apollo Token' as unknown as string[],
    'APLO' as unknown as string[],
    18,
    { gasLimit: gasRequired },
  );
  console.log('aplo token address:', aploAddress);
  const aplo = new Token(aploAddress, deployer, api);
  const { address: usdcAddress } = await tokenFactory.new(
    stableTotalSupply,
    'USD Coin' as unknown as string[],
    'USDC' as unknown as string[],
    6,
    { gasLimit: gasRequired },
  );
  console.log('usdc token address:', usdcAddress);
  const usdc = new Token(usdcAddress, deployer, api);
  const { address: usdtAddress } = await tokenFactory.new(
    stableTotalSupply,
    'Tether USD' as unknown as string[],
    'USDT' as unknown as string[],
    6,
    { gasLimit: gasRequired },
  );
  console.log('usdt token address:', usdtAddress);
  const usdt = new Token(usdtAddress, deployer, api);

  const pairContractRaw = JSON.parse(
    fs.readFileSync(__dirname + `/../artifacts/pair_contract.contract`, 'utf8'),
  );
  const pairAbi = new Abi(pairContractRaw);
  const deployedHash: Hash = await (new Promise(async (resolve, reject) => {
    const unsub = await api.tx.contracts
      .uploadCode(pairAbi.info.source.wasm, null, 'Deterministic')
      .signAndSend(deployer, (result) => {
        if (result.isFinalized) {
          unsub();
          resolve(result.txHash)
        }
        if (result.isError) {
          unsub();
          reject(result)
        }
      });
  }))
  console.log('pair deployed with', deployedHash.toHuman());
  const pairHash = pairAbi.info.source.wasmHash.toHex();

  const factoryContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../artifacts/factory_contract.contract`,
      'utf8',
    ),
  );
  const factoryAbi = new Abi(factoryContractRaw);
  ({ gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: factoryAbi.info.source.wasm },
    factoryAbi.constructors[0].toU8a([deployer.address, pairHash]),
    '',
  ));
  const factoryFactory = new Factory_factory(api, deployer);
  const { address: factoryAddress } = await factoryFactory.new(
    deployer.address,
    pairHash,
    { gasLimit: gasRequired },
  );
  console.log('factory address:', factoryAddress);
  const factory = new Factory(factoryAddress, deployer, api);

  const wnativeContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../artifacts/wnative_contract.contract`,
      'utf8',
    ),
  );
  const wnativeAbi = new Abi(wnativeContractRaw);
  ({ gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: wnativeAbi.info.source.wasm },
    wnativeAbi.constructors[0].toU8a([]),
    '',
  ));

  const wnativeFactory = new Wnative_factory(api, deployer);
  const { address: wnativeAddress } = await wnativeFactory.new({
    gasLimit: gasRequired,
  });
  console.log('wnative address:', wnativeAddress);
  const wnative = new Wnative(wnativeAddress, deployer, api);

  const routerContractRaw = JSON.parse(
    fs.readFileSync(
      __dirname + `/../artifacts/router_contract.contract`,
      'utf8',
    ),
  );
  const routerAbi = new Abi(routerContractRaw);
  ({ gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: routerAbi.info.source.wasm },
    routerAbi.constructors[0].toU8a([factory.address, wnative.address]),
    '',
  ));

  const routerFactory = new Router_factory(api, deployer);
  const { address: routerAddress } = await routerFactory.new(
    factory.address,
    wnative.address,
    { gasLimit: gasRequired },
  );
  console.log('router address:', routerAddress);
  const router = new Router(routerAddress, deployer, api);

  const deadline = '111111111111111111';
  const aploAmount = parseUnits(100).toString();
  const oneSby = parseUnits(1).toString();
  const oneStableCoinAmount = parseUnits(100, 6).toString();
  ({ gasRequired } = await aplo.query.approve(router.address, aploAmount));
  await aplo.tx.approve(router.address, aploAmount, {
    gasLimit: gasRequired,
  });
  ({ gasRequired } = await router.query.addLiquidityNative(
    aplo.address,
    aploAmount,
    aploAmount,
    oneSby,
    deployer.address,
    deadline,
    {
      value: oneSby,
    },
  ));
  await router.tx.addLiquidityNative(
    aplo.address,
    aploAmount,
    aploAmount,
    oneSby,
    deployer.address,
    deadline,
    {
      gasLimit: gasRequired,
      value: oneSby,
    },
  );

  ({ gasRequired } = await usdc.query.approve(
    router.address,
    oneStableCoinAmount,
  ));
  await usdc.tx.approve(router.address, oneStableCoinAmount, {
    gasLimit: gasRequired,
  });
  ({ gasRequired } = await router.query.addLiquidityNative(
    usdc.address,
    oneStableCoinAmount,
    oneStableCoinAmount,
    oneSby,
    deployer.address,
    deadline,
    {
      value: oneSby,
    },
  ));
  await router.tx.addLiquidityNative(
    usdc.address,
    oneStableCoinAmount,
    oneStableCoinAmount,
    oneSby,
    deployer.address,
    deadline,
    {
      gasLimit: gasRequired,
      value: oneSby,
    },
  );

  ({ gasRequired } = await usdt.query.approve(
    router.address,
    oneStableCoinAmount,
  ));
  await usdt.tx.approve(router.address, oneStableCoinAmount, {
    gasLimit: gasRequired,
  });
  ({ gasRequired } = await router.query.addLiquidityNative(
    usdt.address,
    oneStableCoinAmount,
    oneStableCoinAmount,
    oneSby,
    deployer.address,
    deadline,
    {
      value: oneSby,
    },
  ));
  await router.tx.addLiquidityNative(
    usdt.address,
    oneStableCoinAmount,
    oneStableCoinAmount,
    oneSby,
    deployer.address,
    deadline,
    {
      gasLimit: gasRequired,
      value: oneSby,
    },
  );

  const {
    value: { ok: aploSbyAddress },
  } = await factory.query.getPair(aplo.address, wnativeAddress);
  console.log('aploSbyAddress', aploSbyAddress);
  const {
    value: { ok: usdcSbyAddress },
  } = await factory.query.getPair(usdc.address, wnativeAddress);
  console.log('usdcSbyAddress', usdcSbyAddress);
  const {
    value: { ok: usdtSbyAddress },
  } = await factory.query.getPair(usdt.address, wnativeAddress);
  console.log('usdtSbyAddress', usdtSbyAddress);
  await api.disconnect();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

function parseUnits(amount: bigint | number, decimals = 18): bigint {
  return BigInt(amount) * 10n ** BigInt(decimals);
}
