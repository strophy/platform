import { PrivateKey, Transaction } from '@dashevo/dashcore-lib';
import { utils } from '@dashevo/wallet-lib';
import { Platform } from './Platform';

// We're creating a new transaction every time and the index is always 0
const ASSET_LOCK_OUTPUT_INDEX = 0;

/**
 * Creates a funding transaction for the platform identity
 *  and returns one-time key to sign the state transition
 * @param {Platform} this
 * @param {number} fundingAmount - amount of dash to fund the identity's credits
 * @return {Promise<{transaction: Transaction, privateKey: PrivateKey}>}
 *  - transaction and one time private key
 * that can be used to sign registration/top-up state transition
 */
export async function createAssetLockTransaction(
  this : Platform,
  fundingAmount,
): Promise<{ transaction: Transaction, privateKey: PrivateKey, outputIndex: number }> {
  const platform = this;
  const account = await platform.client.getWalletAccount();

  const assetLockOneTimePrivateKey = new PrivateKey();
  const assetLockOneTimePublicKey = assetLockOneTimePrivateKey.toPublicKey();

  const identityAddress = assetLockOneTimePublicKey.toAddress(platform.client.network).toString();

  const changeAddress = account.getUnusedAddress('internal').address;

  const lockTransaction = new Transaction(undefined);

  const output = {
    satoshis: fundingAmount,
    address: identityAddress,
  };

  const utxos = account.getUTXOS();
  const balance = account.getTotalBalance();

  if (balance < output.satoshis) {
    throw new Error(`Not enough balance (${balance}) to cover burn amount of ${fundingAmount}`);
  }

  const selection = utils.coinSelection(utxos, [output]);

  lockTransaction
    .from(selection.utxos)
    // eslint-disable-next-line
    .addBurnOutput(output.satoshis, assetLockOneTimePublicKey._getID())
    .change(changeAddress);

  const utxoAddresses = selection.utxos.map((utxo: any) => utxo.address.toString());

  const utxoHDPrivateKey = account.getPrivateKeys(utxoAddresses);

  // @ts-ignore
  const signingKeys = utxoHDPrivateKey.map((hdprivateKey) => hdprivateKey.privateKey);

  const transaction = lockTransaction.sign(signingKeys);

  return {
    transaction,
    privateKey: assetLockOneTimePrivateKey,
    outputIndex: ASSET_LOCK_OUTPUT_INDEX,
  };
}

export default createAssetLockTransaction;
