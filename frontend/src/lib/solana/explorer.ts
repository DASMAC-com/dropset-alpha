import { CLUSTER, PUBLIC_RPC_URL } from "@/lib/env";

function clusterParam(): string {
  if (CLUSTER === "mainnet") return "";
  if (CLUSTER === "localnet")
    return `?cluster=custom&customUrl=${encodeURIComponent(PUBLIC_RPC_URL)}`;
  return `?cluster=${CLUSTER}`;
}

export function solscanTokenUrl(mint: string): string {
  return `https://solscan.io/token/${mint}${clusterParam()}`;
}

export function solscanTxUrl(signature: string): string {
  return `https://solscan.io/tx/${signature}${clusterParam()}`;
}

export function solscanAccountUrl(address: string): string {
  return `https://solscan.io/account/${address}${clusterParam()}`;
}
