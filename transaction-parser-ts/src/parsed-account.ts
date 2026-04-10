import type { Address } from "@solana/kit";

export type ResolvedAccount = {
  address: Address;
  writable: boolean;
  signer: boolean;
};

export type MessageHeader = {
  numRequiredSignatures: number;
  numReadonlySignedAccounts: number;
  numReadonlyUnsignedAccounts: number;
};

export type LoadedAddresses = {
  writable: readonly Address[];
  readonly: readonly Address[];
};

/**
 * Resolves all accounts in a transaction into a flat array with signer/writable metadata.
 *
 * Static account keys are categorized using the message header boundaries:
 *
 * ```
 * 0 ---- writable signers ---- a ---- readonly signers ---- b ---- writable non-signers ---- c ---- readonly non-signers ---- n
 * ```
 *
 * ALT-loaded addresses are appended after static keys (writable first, then readonly).
 * ALT-loaded addresses are never signers.
 */
export function resolveAccounts(
  accountKeys: readonly Address[],
  header: MessageHeader,
  loadedAddresses?: LoadedAddresses,
): ResolvedAccount[] {
  const nSigners = header.numRequiredSignatures;
  const roSigners = header.numReadonlySignedAccounts;
  const roNonSigners = header.numReadonlyUnsignedAccounts;

  const a = nSigners - roSigners;
  const b = nSigners;
  const n = accountKeys.length;
  const c = n - roNonSigners;

  const accounts: ResolvedAccount[] = accountKeys.map((address, i) => {
    let writable: boolean;
    let signer: boolean;

    if (i < a) {
      writable = true;
      signer = true;
    } else if (i < b) {
      writable = false;
      signer = true;
    } else if (i < c) {
      writable = true;
      signer = false;
    } else {
      writable = false;
      signer = false;
    }

    return { address, writable, signer };
  });

  if (loadedAddresses) {
    for (const address of loadedAddresses.writable) {
      accounts.push({ address, writable: true, signer: false });
    }
    for (const address of loadedAddresses.readonly) {
      accounts.push({ address, writable: false, signer: false });
    }
  }

  return accounts;
}
