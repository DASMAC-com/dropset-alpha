import {
  type Address,
  type ClusterUrl,
  createSolanaRpc,
  getAddressEncoder,
  getProgramDerivedAddress,
  type ProgramDerivedAddressBump,
} from "@solana/kit";
import type { createHttpTransport } from "@solana/rpc-transport-http";

import { LOCALNET_URL, MARKET_SEED_STR } from "@/ts-sdk/const";
import {
  DROPSET_PROGRAM_ADDRESS,
  getMarketAccountDecoder,
} from "@/ts-sdk/generated";
import { toMarketViewAll } from "../dropset-interface";
import type {
  DropsetMarketAccount,
  DropsetMarketView,
  Flatten,
} from "../types";

export type RpcClient = NonNullable<ReturnType<typeof getRpcClient>>;

type HttpTransportConfig = Flatten<Parameters<typeof createHttpTransport>[0]>;

type RpcClientArgs = {
  clusterUrl?: ClusterUrl;
  /** See: {@link createSolanaRpc} */
  config?: HttpTransportConfig;
};

/**
 * Creates a Solana RPC client with {@link createSolanaRpc}.
 *
 * Defaults to a localnet RPC.
 */
export function getRpcClient(args?: RpcClientArgs) {
  const { clusterUrl, config } = args ?? {};
  const rpc = createSolanaRpc(clusterUrl ?? LOCALNET_URL, config);

  return rpc;
}

/**
 * Fetches the dropset market accounts owned by the dropset program.
 */
export async function fetchDropsetMarketAccounts(
  rpcClient: RpcClient,
): Promise<DropsetMarketAccount[]> {
  const markets = await rpcClient
    .getProgramAccounts(DROPSET_PROGRAM_ADDRESS, { encoding: "base64" })
    .send();

  const decoder = getMarketAccountDecoder();

  return markets.map(
    (market) =>
      ({
        address: market.pubkey,
        ...decoder.decode(Buffer.from(market.account.data[0], "base64")),
      }) as const,
  );
}

/**
 * Fetches a single dropset market account.
 */
export async function fetchDropsetMarketAccount(
  rpcClient: RpcClient,
  marketAddress: Address,
): Promise<DropsetMarketAccount | undefined> {
  const market = await rpcClient
    .getAccountInfo(marketAddress, { encoding: "base64" })
    .send();

  const decoder = getMarketAccountDecoder();

  if (!market.value) {
    return undefined;
  }

  return {
    address: marketAddress,
    ...decoder.decode(Buffer.from(market.value.data[0], "base64")),
  };
}

/**
 * Fetches the dropset market accounts owned by the dropset program and converts them into
 * more ergonomic {@link DropsetMarketView} types.
 */
export async function fetchDropsetMarketViews(
  rpcClient: RpcClient,
): Promise<DropsetMarketView[]> {
  return (await fetchDropsetMarketAccounts(rpcClient)).map(
    ({ address, ...market }) => ({
      address,
      ...toMarketViewAll(market),
    }),
  );
}

/**
 * Fetches a single dropset market account and converts it into a more ergonomic
 * {@link DropsetMarketView} type.
 */
export async function fetchDropsetMarketView(
  rpcClient: RpcClient,
  marketAddress: Address,
): Promise<DropsetMarketView | undefined> {
  const account = await fetchDropsetMarketAccount(rpcClient, marketAddress);
  if (!account) return undefined;

  return {
    address: marketAddress,
    ...toMarketViewAll(account),
  };
}

/**
 * Gets the derived market address given the base mint, quote mint, and dropset program
 * {@link Address}es.
 */
export async function deriveMarketAddress(
  baseMint: Address,
  quoteMint: Address,
  dropsetProgramAddress?: Address,
): Promise<readonly [Address<string>, ProgramDerivedAddressBump]> {
  const addressEncoder = getAddressEncoder();
  return await getProgramDerivedAddress({
    programAddress: dropsetProgramAddress ?? DROPSET_PROGRAM_ADDRESS,
    seeds: [
      addressEncoder.encode(baseMint),
      addressEncoder.encode(quoteMint),
      MARKET_SEED_STR,
    ],
  });
}

export * from "./liquidity";
