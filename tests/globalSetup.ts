import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
// Create a new instance of contract
const wsProvider = new WsProvider('ws://127.0.0.1:9944');
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });
export default async function setupApi(): Promise<void> {
  const api = await ApiPromise.create({ provider: wsProvider });
  const alice = keyring.addFromUri('//Alice');
  const bob = keyring.addFromUri('//Bob');
  globalThis.setup = { api, alice, bob };
}
