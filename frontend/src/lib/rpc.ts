import { createSolanaRpc, devnet, testnet } from "@solana/kit";
import { CLUSTER, getRpcFromEnv, RPC_URL } from "./env";

export const rpcClient = getRpcFromEnv();

/**
 *  Return the RPC client typed as a test network client. This facilitates airdrops and other
 * test network APIs.
 */
export function getTestRpcClient() {
  switch (CLUSTER) {
    case "localnet":
      return createSolanaRpc(devnet(RPC_URL));
    case "devnet":
      return createSolanaRpc(devnet(RPC_URL));
    case "testnet":
      return createSolanaRpc(testnet(RPC_URL));
    default:
      throw new Error("Test RPC methods are not available on mainnet");
  }
}
