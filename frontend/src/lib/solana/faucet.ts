import { getTransactionDecoder } from "@solana/kit";

/**
 * The Rust faucet's serde JSON is the transaction wire bytes expressed as a
 * JSON tree. Counts, prefixes, and all encoding are already present. Simply
 * recursively flatten every number out of the structure. Fields are guaranteed
 * to be in the same order as the Rust struct prior to JSON serialization.
 */
type SerdeWireValue =
  | number
  | SerdeWireValue[]
  | { [key: string]: SerdeWireValue };

function flattenWireBytes(value: SerdeWireValue): number[] {
  if (typeof value === "number") return [value];
  if (Array.isArray(value)) return value.flatMap(flattenWireBytes);
  return Object.values(value).flatMap(flattenWireBytes);
}

export type MintRequest = {
  address: string;
  is_base: boolean;
  amount?: number;
};

/**
 * Request tokens from the faucet. Returns the partially-signed transaction
 * as wire bytes ready for the wallet to co-sign.
 */
export async function requestFaucetTransaction(req: MintRequest) {
  const res = await fetch("/api/faucet", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error ?? "Faucet request failed");
  }

  const { transaction }: { transaction: SerdeWireValue } = await res.json();
  const bytes = new Uint8Array(flattenWireBytes(transaction));

  const tx = getTransactionDecoder().decode(bytes);
  return tx;
}
