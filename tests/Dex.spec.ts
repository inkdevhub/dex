import { expect, use } from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { encodeAddress } from '@polkadot/keyring';
import BN from 'bn.js';
import Factory_factory from '../types/constructors/factory_contract';
import Pair_factory from '../types/constructors/pair_contract';
import Token_factory from '../types/constructors/psp22_token';
import Wnative_factory from '../types/constructors/wnative_contract';
import Router_factory from '../types/constructors/router_contract';
import Factory from '../types/contracts/factory_contract';
import Pair from '../types/contracts/pair_contract';
import Token from '../types/contracts/psp22_token';
import Wnative from '../types/contracts/wnative_contract';
import Router from '../types/contracts/router_contract';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { AccountId, Hash } from 'types-arguments/factory_contract';
import { ReturnNumber } from '@supercolony/typechain-types';

use(chaiAsPromised);

const zeroAddress = encodeAddress(
  '0x0000000000000000000000000000000000000000000000000000000000000000',
);
const MINIMUM_LIQUIDITY = 1000;

// Create a new instance of contract
const wsProvider = new WsProvider('ws://127.0.0.1:9944');
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

describe('Dex spec', () => {
  let pairFactory: Pair_factory;
  let factoryFactory: Factory_factory;
  let routerFactory: Router_factory;
  let tokenFactory: Token_factory;
  let wnativeFactory: Wnative_factory;
  let api: ApiPromise;
  let deployer: KeyringPair;
  let wallet: KeyringPair;
  // const alice = keyring.addFromUri('//Alice//stash');
  // const bob = keyring.addFromUri('//Bob//stash');

  let pairHash: Hash;
  let factory: Factory;
  let router: Router;
  let [token0, token1]: Token[] = [];
  let wnative: Wnative;

  let gasRequired: bigint;

  async function setup(): Promise<void> {
    api = await ApiPromise.create({ provider: wsProvider });
    deployer = keyring.addFromUri('//Alice');
    wallet = keyring.addFromUri('//Bob');
    pairFactory = new Pair_factory(api, deployer);
    const pair = new Pair((await pairFactory.new()).address, deployer, api);
    pairHash = pair.abi.info.source.wasmHash.toHex();
    factoryFactory = new Factory_factory(api, deployer);
    factory = new Factory(
      (await factoryFactory.new(wallet.address, pairHash)).address,
      deployer,
      api,
    );
  }

  async function setupPsp22(): Promise<void> {
    tokenFactory = new Token_factory(api, deployer);
    const totalSupply = new BN(10000000);

    const tokenAaddress = (await tokenFactory.new(totalSupply)).address;
    const tokenBaddress = (await tokenFactory.new(totalSupply)).address;
    const [token0Address, token1Address] =
      tokenAaddress > tokenBaddress
        ? [tokenBaddress, tokenAaddress]
        : [tokenAaddress, tokenBaddress];
    token0 = new Token(token0Address, deployer, api);
    token1 = new Token(token1Address, deployer, api);
  }

  async function setupRouter(): Promise<void> {
    wnativeFactory = new Wnative_factory(api, deployer);
    wnative = new Wnative((await wnativeFactory.new()).address, deployer, api);
    routerFactory = new Router_factory(api, deployer);
    router = new Router(
      (
        await routerFactory.new(factory.address, wnative.address, pairHash)
      ).address,
      deployer,
      api,
    );
  }

  it('feeTo, feeToSetter, allPairsLength', async () => {
    await setup();
    expect((await factory.query.feeTo()).value).to.equal(zeroAddress);
    expect((await factory.query.feeToSetter()).value).to.equal(wallet.address);
    expect((await factory.query.allPairsLength()).value).to.equal(0);
  });

  it('set fee', async () => {
    await setupPsp22();
    expect((await factory.query.feeTo()).value).to.equal(zeroAddress);
    await expect(factory.tx.setFeeTo(token0.address)).to.eventually.be.rejected;
    const { gasRequired } = await factory
      .withSigner(wallet)
      .query.setFeeTo(token0.address);
    await factory
      .withSigner(wallet)
      .tx.setFeeTo(token0.address, { gasLimit: gasRequired });
    expect((await factory.query.feeTo()).value).to.equal(token0.address);
  });

  it('set fee setter', async () => {
    expect((await factory.query.feeToSetter()).value).to.equal(wallet.address);
    await expect(factory.tx.setFeeToSetter(token0.address)).to.eventually.be
      .rejected;
    const { gasRequired } = await factory
      .withSigner(wallet)
      .query.setFeeToSetter(token0.address);
    await factory
      .withSigner(wallet)
      .tx.setFeeToSetter(token0.address, { gasLimit: gasRequired });
    expect((await factory.query.feeToSetter()).value).to.equal(token0.address);
  });

  it('create pair', async () => {
    expect((await factory.query.allPairsLength()).value).to.equal(0);
    const expectedAddress = (
      await factory.query.createPair(token0.address, token1.address)
    ).value.ok;
    expect(expectedAddress).not.equal(zeroAddress);
    const { gasRequired } = await factory.query.createPair(
      token0.address,
      token1.address,
    );
    const result = await factory.tx.createPair(token0.address, token1.address, {
      gasLimit: gasRequired,
    });
    emit(result, 'PairCreated', {
      token0: token0.address,
      token1: token1.address,
      pair: expectedAddress,
      pairLen: 1,
    });
    expect((await factory.query.allPairsLength()).value).to.equal(1);
  });

  let pair: Pair;
  it('can mint pair', async () => {
    const liqudity = 10000;
    const pairAddress = await factory.query.getPair(
      token0.address,
      token1.address,
    );
    pair = new Pair(pairAddress.value as string, deployer, api);
    gasRequired = (await token0.query.transfer(
      pair.address,
      liqudity,
      [],
    )).gasRequired;
    await token0.tx.transfer(pair.address, liqudity, [], {
      gasLimit: gasRequired,
    });
    await token1.tx.transfer(pair.address, liqudity, [], {
      gasLimit: gasRequired,
    });
    expect(
      (await pair.query.balanceOf(wallet.address)).value.toNumber(),
    ).to.equal(0);
    gasRequired = (await pair.query.mint(wallet.address)).gasRequired;
    const result = await pair.tx.mint(wallet.address, { gasLimit: gasRequired });
    emit(result, 'Mint', {
      sender: deployer.address,
      amount0: liqudity,
      amount1: liqudity,
    });
    expect(
      (await pair.query.balanceOf(wallet.address)).value.toNumber(),
    ).to.equal(liqudity - MINIMUM_LIQUIDITY);
  });

  it('can swap tokens', async () => {
    const token1Amount = 1020;
    gasRequired = (await token0.query.transfer(pair.address, token1Amount, [])).gasRequired;
    await token0.tx.transfer(pair.address, token1Amount, [], {
      gasLimit: gasRequired,
    });
    expect(
      (await token1.query.balanceOf(wallet.address)).value.toNumber(),
    ).to.equal(0);
    gasRequired = (await pair.query.swap(0, 900, wallet.address)).gasRequired;
    const result = await pair.tx.swap(0, 900, wallet.address, {
      gasLimit: gasRequired,
    });
    emit(result, 'Swap', {
      sender: deployer.address,
      amount0In: token1Amount,
      amount1In: 0,
      amount0Out: 0,
      amount1Out: 900,
      to: wallet.address,
    });
    expect(
      (await token1.query.balanceOf(wallet.address)).value.toNumber(),
    ).to.equal(900);
  });

  it('can burn LP token', async () => {
    const beforeToken1Balance = (await token0.query.balanceOf(wallet.address))
      .value.rawNumber;
    const beforeToken2Balance = (await token1.query.balanceOf(wallet.address))
      .value.rawNumber;
    gasRequired = (await pair
      .withSigner(wallet)
      .query.transfer(pair.address, 2000, [])).gasRequired;
    await pair
      .withSigner(wallet)
      .tx.transfer(pair.address, 2000, [], { gasLimit: gasRequired });
    gasRequired = (await pair
      .withSigner(wallet)
      .query.burn(wallet.address)).gasRequired;
    const result = await pair
      .withSigner(wallet)
      .tx.burn(wallet.address, { gasLimit: gasRequired });
    const lockedToken1Balance = 2204;
    const lockedToken2Balance = 1820;
    emit(result, 'Burn', {
      sender: wallet.address,
      amount0: lockedToken1Balance,
      amount1: lockedToken2Balance,
      to: wallet.address,
    });
    expect(
      (await token0.query.balanceOf(wallet.address)).value.rawNumber.sub(
        beforeToken1Balance,
      ),
    ).to.eql(new BN(lockedToken1Balance));
    expect(
      (await token1.query.balanceOf(wallet.address)).value.rawNumber.sub(
        beforeToken2Balance,
      ),
    ).to.eql(new BN(lockedToken2Balance));
  });

  it('can add liqudity via router', async () => {
    await setupRouter();
    const deadline = '111111111111111111';
    gasRequired = (await token0.query.approve(router.address, 10000)).gasRequired;
    await token0.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });
    gasRequired = (await router.query.addLiquidityNative(
      token0.address,
      10000,
      10000,
      10000,
      deployer.address,
      deadline,
      {
        value: 10000,
      },
    )).gasRequired;
    await router.tx.addLiquidityNative(
      token0.address,
      10000,
      10000,
      10000,
      deployer.address,
      deadline,
      {
        gasLimit: gasRequired,
        value: 10000,
      },
    );
    expect((await factory.query.allPairsLength()).value).to.equal(2);
  });

  it('can swapExactNativeForTokens via router', async () => {
    const deadline = '111111111111111111';
    const { gasRequired } = await router.query.swapExactNativeForTokens(
      1000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
      {
        value: 10000,
      },
    );
    await router.tx.swapExactNativeForTokens(
      1000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
      {
        gasLimit: gasRequired,
        value: 10000,
      },
    );
  });

  it('can swapNativeForExactTokens via router', async () => {
    const deadline = '111111111111111111';
    const { gasRequired } = await router.query.swapNativeForExactTokens(
      1000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
      {
        value: 10000,
      },
    );
    await router.tx.swapNativeForExactTokens(
      1000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
      {
        gasLimit: gasRequired,
        value: 10000,
      },
    );
  });

  it('can swapExactTokensForTokens via router', async () => {
    const deadline = '111111111111111111';
    gasRequired = (await wnative.query.deposit({ value: 10000 })).gasRequired;
    await wnative.tx.deposit({ gasLimit: gasRequired, value: 10000 });
    gasRequired = (await wnative.query.approve(router.address, 10000)).gasRequired;
    await wnative.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });

    gasRequired = (await router.query.swapExactTokensForTokens(
      10000,
      1000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
    )).gasRequired;

    await router.tx.swapExactTokensForTokens(
      10000,
      1000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
      { gasLimit: gasRequired },
    );
  });

  it('can swapTokensForExactTokens via router', async () => {
    const deadline = '111111111111111111';
    gasRequired = (await wnative.query.deposit({ value: 100000 })).gasRequired;
    await wnative.tx.deposit({ gasLimit: gasRequired, value: 100000 });
    await wnative.tx.approve(router.address, 100000, {
      gasLimit: gasRequired,
    });
    gasRequired = (await router.query.swapTokensForExactTokens(
      1000,
      100000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
    )).gasRequired;
    await router.tx.swapTokensForExactTokens(
      1000,
      100000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
      { gasLimit: gasRequired },
    );
  });

  it('can add liqudity more via router', async () => {
    const deadline = '111111111111111111';
    gasRequired = (await token0.query.approve(router.address, 10000)).gasRequired;
    await token0.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });
    const balance = await getBalance(deployer.address);
    gasRequired = (await router.query.addLiquidityNative(
      token0.address,
      10000,
      0,
      0,
      deployer.address,
      deadline,
      {
        value: 1000000000000000,
      },
    )).gasRequired;
    await router.tx.addLiquidityNative(
      token0.address,
      10000,
      0,
      0,
      deployer.address,
      deadline,
      {
        gasLimit: gasRequired,
        value: 1000000000000000,
      },
    )
    const afterBalance = await getBalance(deployer.address);
    expect(balance.sub(afterBalance).toNumber()).lt(1000000000000000);
    expect((await factory.query.allPairsLength()).value).to.equal(2);
  });

  it('can remove liqudity via router', async () => {
    const deadline = '111111111111111111';
    gasRequired = (await token0.query.approve(router.address, 10000)).gasRequired;
    await token0.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });
    const lpToken = new Pair(
      (
        await factory.query.getPair(wnative.address, token0.address)
      ).value.toString(),
      deployer,
      api,
    );
    await lpToken.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });
    const balance = await getBalance(wallet.address);
    gasRequired = (await router.query.removeLiquidityNative(
      token0.address,
      10000,
      0,
      0,
      wallet.address,
      deadline,
    )).gasRequired;
    await router.tx.removeLiquidityNative(
      token0.address,
      10000,
      0,
      0,
      wallet.address,
      deadline,
      { gasLimit: gasRequired },
    );
    const afterBalance = await getBalance(wallet.address);
    expect(afterBalance.sub(balance).toNumber()).gt(10000);
    expect((await factory.query.allPairsLength()).value).to.equal(2);
  });
  async function getBalance(address: AccountId): Promise<BN> {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return ((await api.query.system.account(address)) as any).data.free;
  }
});

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function emit(result: { events?: any }, name: string, args: any): void {
  const event = result.events.find(
    (event: { name: string }) => event.name === name,
  );
  for (const key of Object.keys(event.args)) {
    if (event.args[key] instanceof ReturnNumber) {
      event.args[key] = event.args[key].toNumber();
    }
  }
  expect(event).eql({
    name,
    args,
  });
}
