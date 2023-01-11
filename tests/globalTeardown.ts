export default async function teardown(): Promise<void> {
  await globalThis.setup.api.disconnect();
}
