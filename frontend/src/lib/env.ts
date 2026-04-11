import { type ClusterUrl, devnet, mainnet, testnet } from "@solana/rpc-types";
import { getRpcClient, type RpcClient } from "@/ts-sdk";

export type Cluster = "localnet" | "devnet" | "testnet" | "mainnet";

const VALID_CLUSTERS: Set<string> = new Set([
  "localnet",
  "devnet",
  "testnet",
  "mainnet",
]);

function loadCluster(): Cluster {
  const raw = process.env.NEXT_PUBLIC_CLUSTER;
  if (!raw) {
    throw new Error("NEXT_PUBLIC_CLUSTER is not set");
  }
  if (!VALID_CLUSTERS.has(raw)) {
    throw new Error(
      `NEXT_PUBLIC_CLUSTER="${raw}" is not valid. Expected: ${[...VALID_CLUSTERS].join(", ")}`,
    );
  }
  return raw as Cluster;
}

export const CLUSTER: Cluster = loadCluster();

const PUBLIC_RPC_URLS = {
  localnet: devnet("http://localhost:8899"),
  devnet: devnet("https://api.devnet.solana.com"),
  testnet: testnet("https://api.testnet.solana.com"),
  mainnet: mainnet("https://api.mainnet-beta.solana.com"),
} satisfies Record<Cluster, ClusterUrl>;

const PUBLIC_WS_URLS: Record<Cluster, string> = {
  localnet: "ws://localhost:8900",
  devnet: "wss://api.devnet.solana.com",
  testnet: "wss://api.testnet.solana.com",
  mainnet: "wss://api.mainnet-beta.solana.com",
};

const maybePutative = (
  fn: typeof devnet | typeof testnet | typeof mainnet,
  url?: string,
) => (url ? fn(url) : undefined);

const PRIVATE_RPC_URLS = {
  localnet: maybePutative(devnet, process.env.SERVER_LOCALNET_RPC_URL),
  devnet: maybePutative(devnet, process.env.SERVER_DEVNET_RPC_URL),
  testnet: maybePutative(testnet, process.env.SERVER_TESTNET_RPC_URL),
  mainnet: maybePutative(mainnet, process.env.SERVER_MAINNET_RPC_URL),
} satisfies Record<Cluster, ClusterUrl | undefined>;

const PRIVATE_WS_URLS: Record<Cluster, string | undefined> = {
  localnet: process.env.SERVER_LOCALNET_WS_URL,
  devnet: process.env.SERVER_DEVNET_WS_URL,
  testnet: process.env.SERVER_TESTNET_WS_URL,
  mainnet: process.env.SERVER_MAINNET_WS_URL,
};

/** Public RPC URL for the current cluster. Safe to expose to the browser. */
export const PUBLIC_RPC_URL: ClusterUrl = PUBLIC_RPC_URLS[CLUSTER];

/** Public RPC URL for the current cluster. Safe to expose to the browser. */
export const PUBLIC_WS_URL: string = PUBLIC_WS_URLS[CLUSTER];

/** Private RPC URL if set, otherwise falls back to public. */
export const RPC_URL: ClusterUrl = PRIVATE_RPC_URLS[CLUSTER] ?? PUBLIC_RPC_URL;

/** Private WS URL if set, otherwise falls back to public. */
export const WS_URL: ClusterUrl = PRIVATE_WS_URLS[CLUSTER] ?? RPC_URL;

/**
 * Creates an RPC client using the private RPC URL if available, public otherwise.
 *
 * Since `SERVER_*` env vars are only available server-side (no `NEXT_PUBLIC_` prefix),
 * this naturally splits behavior: server components get the paid/private RPC if it exists,
 * client components fall back to the public endpoint.
 */
export function getRpcFromEnv(rpc?: RpcClient): RpcClient {
  return rpc ?? getRpcClient({ clusterUrl: RPC_URL });
}

/** Server-side faucet URL. Not exposed to the browser. */
export const FAUCET_URL = process.env.FAUCET_URL ?? "http://localhost:9090";
