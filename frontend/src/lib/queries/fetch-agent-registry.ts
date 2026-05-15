import type { Address } from "@solana/addresses";

export type AgentRegistryEntry = {
  name: string;
  kind: "maker" | "taker";
  pubkey: Address;
};

export async function fetchAgentRegistry(): Promise<AgentRegistryEntry[]> {
  const res = await fetch("/api/agents");
  if (!res.ok) return [];
  return (await res.json()) as AgentRegistryEntry[];
}
