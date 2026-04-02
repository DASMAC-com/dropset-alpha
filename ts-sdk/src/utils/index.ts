import { type ClusterUrl, createSolanaRpc } from "@solana/kit";
import type { createHttpTransport } from "@solana/rpc-transport-http";

import { LOCALNET_URL } from "@/const";
import { DROPSET_PROGRAM_ADDRESS, getMarketAccountDecoder } from "@/generated";
import type { Flatten } from "../types";

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
 * Gets the dropset market accounts owned by the dropset program.
 */
export async function getDropsetMarkets(
  rpcClient: ReturnType<typeof getRpcClient>,
) {
  rpcClient;
  const markets = await rpcClient
    .getProgramAccounts(DROPSET_PROGRAM_ADDRESS, { encoding: "base64" })
    .send();

  const decoder = getMarketAccountDecoder();

  return markets.map(
    (market) =>
      [
        market.pubkey,
        decoder.decode(Buffer.from(market.account.data[0], "base64")),
      ] as const,
  );
}
