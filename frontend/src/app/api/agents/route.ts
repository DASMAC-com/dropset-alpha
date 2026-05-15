import fs from "node:fs/promises";
import path from "node:path";
import { NextResponse } from "next/server";

export type AgentRegistryEntry = {
  name: string;
  kind: "maker" | "taker";
  pubkey: string;
};

/**
 * Returns the trader registry written by `services/shared/examples/initialization_helper.rs`.
 *
 * The frontend uses this to label each fill in the transaction log with the
 * personality (maker, retail-1, whale-1, etc.) that submitted the trade.
 *
 * Returns an empty array when the file is missing — e.g. when the frontend is
 * run against devnet/testnet/mainnet rather than the local helper script.
 */
export async function GET() {
  const filePath = path.join(
    process.cwd(),
    "..",
    "services",
    "taker-bot",
    "agents.json",
  );
  try {
    const raw = await fs.readFile(filePath, "utf8");
    const parsed = JSON.parse(raw) as AgentRegistryEntry[];
    return NextResponse.json(parsed);
  } catch {
    return NextResponse.json([]);
  }
}
