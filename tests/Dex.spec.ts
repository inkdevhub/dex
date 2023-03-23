import { expect } from '@jest/globals';
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
import { AccountId, Hash } from 'types-arguments/factory_contract';
import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { emit, revertedWith } from './testHelpers';
import type { WeightV2 } from '@polkadot/types/interfaces';

const zeroAddress = encodeAddress(
  '0x0000000000000000000000000000000000000000000000000000000000000000',
);
const MINIMUM_LIQUIDITY = 1000;

describe('Dex spec', () => {
  let api: ApiPromise;
  let deployer: KeyringPair;
  let wallet: KeyringPair;

  let pairFactory: Pair_factory;
  let factoryFactory: Factory_factory;
  let routerFactory: Router_factory;
  let tokenFactory: Token_factory;
  let wnativeFactory: Wnative_factory;

  let pairHash: Hash;
  let factory: Factory;
  let router: Router;
  let [token0, token1]: Token[] = [];
  let wnative: Wnative;

  let gasRequired: WeightV2;

  async function setup(): Promise<void> {
    ({ api, alice: deployer, bob: wallet } = globalThis.setup);
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

    const tokenAaddress = (
      await tokenFactory.new(
        totalSupply,
        'TOKEN_A' as unknown as string[],
        'TKNA' as unknown as string[],
        18,
      )
    ).address;
    const tokenBaddress = (
      await tokenFactory.new(
        totalSupply,
        'TOKEN_B' as unknown as string[],
        'TKNB' as unknown as string[],
        18,
      )
    ).address;
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
      (await routerFactory.new(factory.address, wnative.address)).address,
      deployer,
      api,
    );
  }

  it('feeTo, feeToSetter, allPairsLength', async () => {
    await setup();
    expect((await factory.query.feeTo()).value.ok).toBe(zeroAddress);
    expect((await factory.query.feeToSetter()).value.ok).toBe(wallet.address);
    expect((await factory.query.allPairsLength()).value.ok).toBe(0);
  });

  it('set fee', async () => {
    await setupPsp22();
    expect((await factory.query.feeTo()).value.ok).toBe(zeroAddress);
    revertedWith(
      await factory.query.setFeeTo(token0.address),
      'callerIsNotFeeSetter',
    );
    ({ gasRequired } = await factory
      .withSigner(wallet)
      .query.setFeeTo(token0.address));
    await factory
      .withSigner(wallet)
      .tx.setFeeTo(token0.address, { gasLimit: gasRequired });
    expect((await factory.query.feeTo()).value.ok).toBe(token0.address);
  });

  it('set fee setter', async () => {
    expect((await factory.query.feeToSetter()).value.ok).toBe(wallet.address);
    revertedWith(
      await factory.query.setFeeToSetter(token0.address),
      'callerIsNotFeeSetter',
    );
    ({ gasRequired } = await factory
      .withSigner(wallet)
      .query.setFeeToSetter(token0.address));
    await factory
      .withSigner(wallet)
      .tx.setFeeToSetter(token0.address, { gasLimit: gasRequired });
    expect((await factory.query.feeToSetter()).value.ok).toBe(token0.address);
  });

  it('create pair', async () => {
    expect((await factory.query.allPairsLength()).value.ok).toBe(0);
    const {
      gasRequired,
      value: {
        ok: { ok: expectedAddress },
      },
    } = await factory.query.createPair(token0.address, token1.address);
    expect(expectedAddress).not.toBe(zeroAddress);
    const result = await factory.tx.createPair(token0.address, token1.address, {
      gasLimit: gasRequired,
    });
    emit(result, 'PairCreated', {
      token0: token0.address,
      token1: token1.address,
      pair: expectedAddress,
      pairLen: 1,
    });
    expect((await factory.query.allPairsLength()).value.ok).toBe(1);
  });

  let pair: Pair;
  it('can mint pair', async () => {
    const liqudity = 10000;
    const pairAddress = await factory.query.getPair(
      token0.address,
      token1.address,
    );
    pair = new Pair(pairAddress.value.ok as string, deployer, api);
    ({ gasRequired } = await token0.query.transfer(pair.address, liqudity, []));
    await token0.tx.transfer(pair.address, liqudity, [], {
      gasLimit: gasRequired,
    });
    await token1.tx.transfer(pair.address, liqudity, [], {
      gasLimit: gasRequired,
    });
    expect(
      (await pair.query.balanceOf(wallet.address)).value.ok.toNumber(),
    ).toBe(0);
    ({ gasRequired } = await pair.query.mint(wallet.address));
    const result = await pair.tx.mint(wallet.address, {
      gasLimit: gasRequired,
    });
    emit(result, 'Mint', {
      sender: deployer.address,
      amount0: liqudity,
      amount1: liqudity,
    });
    expect(
      (await pair.query.balanceOf(wallet.address)).value.ok.toNumber(),
    ).toBe(liqudity - MINIMUM_LIQUIDITY);
  });

  it('can swap tokens', async () => {
    const token1Amount = 1020;
    ({ gasRequired } = await token0.query.transfer(
      pair.address,
      token1Amount,
      [],
    ));
    await token0.tx.transfer(pair.address, token1Amount, [], {
      gasLimit: gasRequired,
    });
    expect(
      (await token1.query.balanceOf(wallet.address)).value.ok.toNumber(),
    ).toBe(0);
    ({ gasRequired } = await pair.query.swap(0, 900, wallet.address));
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
      (await token1.query.balanceOf(wallet.address)).value.ok.toNumber(),
    ).toBe(900);
  });

  it('can burn LP token', async () => {
    const beforeToken1Balance = (await token0.query.balanceOf(wallet.address))
      .value.ok.rawNumber;
    const beforeToken2Balance = (await token1.query.balanceOf(wallet.address))
      .value.ok.rawNumber;
    ({ gasRequired } = await pair
      .withSigner(wallet)
      .query.transfer(pair.address, 2000, []));
    await pair
      .withSigner(wallet)
      .tx.transfer(pair.address, 2000, [], { gasLimit: gasRequired });
    ({ gasRequired } = await pair
      .withSigner(wallet)
      .query.burn(wallet.address));
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
      (await token0.query.balanceOf(wallet.address)).value.ok.rawNumber.sub(
        beforeToken1Balance,
      ),
    ).toEqual(new BN(lockedToken1Balance));
    expect(
      (await token1.query.balanceOf(wallet.address)).value.ok.rawNumber.sub(
        beforeToken2Balance,
      ),
    ).toEqual(new BN(lockedToken2Balance));
  });

  it('can add liqudity via router', async () => {
    await setupRouter();
    const deadline = '111111111111111111';
    ({ gasRequired } = await token0.query.approve(router.address, 10000));
    await token0.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });
    ({ gasRequired } = await router.query.addLiquidityNative(
      token0.address,
      10000,
      10000,
      10000,
      deployer.address,
      deadline,
      {
        value: 10000,
      },
    ));
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
    expect((await factory.query.allPairsLength()).value.ok).toBe(2);
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
    ({ gasRequired } = await wnative.query.deposit({ value: 10000 }));
    await wnative.tx.deposit({ gasLimit: gasRequired, value: 10000 });
    ({ gasRequired } = await wnative.query.approve(router.address, 10000));
    await wnative.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });

    ({ gasRequired } = await router.query.swapExactTokensForTokens(
      10000,
      1000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
    ));

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
    ({ gasRequired } = await wnative.query.deposit({ value: 100000 }));
    await wnative.tx.deposit({ gasLimit: gasRequired, value: 100000 });
    await wnative.tx.approve(router.address, 100000, {
      gasLimit: gasRequired,
    });
    ({ gasRequired } = await router.query.swapTokensForExactTokens(
      1000,
      100000,
      [wnative.address, token0.address],
      wallet.address,
      deadline,
    ));
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
    ({ gasRequired } = await token0.query.approve(router.address, 10000));
    await token0.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });
    const balance = await getBalance(deployer.address);
    ({ gasRequired } = await router.query.addLiquidityNative(
      token0.address,
      10000,
      0,
      0,
      deployer.address,
      deadline,
      {
        value: 1000000000000000,
      },
    ));
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
    );
    const afterBalance = await getBalance(deployer.address);
    expect(balance.sub(afterBalance).toNumber()).toBeLessThan(1000000000000000);
    expect((await factory.query.allPairsLength()).value.ok).toBe(2);
  });

  it('can remove liqudity via router', async () => {
    const deadline = '111111111111111111';
    ({ gasRequired } = await token0.query.approve(router.address, 10000));
    await token0.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });
    const lpToken = new Pair(
      (
        await factory.query.getPair(wnative.address, token0.address)
      ).value.ok.toString(),
      deployer,
      api,
    );
    await lpToken.tx.approve(router.address, 10000, {
      gasLimit: gasRequired,
    });
    const balance = await getBalance(wallet.address);
    ({ gasRequired } = await router.query.removeLiquidityNative(
      token0.address,
      10000,
      0,
      0,
      wallet.address,
      deadline,
    ));
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
    expect(afterBalance.sub(balance).toNumber()).toBeGreaterThan(10000);
    expect((await factory.query.allPairsLength()).value.ok).toBe(2);
  });

  async function getBalance(address: AccountId): Promise<BN> {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return ((await api.query.system.account(address)) as any).data.free;
  }
});
