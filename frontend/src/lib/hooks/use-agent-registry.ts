"use client";

import type { Address } from "@solana/addresses";
import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import {
  type AgentRegistryEntry,
  fetchAgentRegistry,
} from "@/lib/queries/fetch-agent-registry";

export function useAgentRegistry() {
  const query = useQuery({
    queryKey: ["agent-registry"],
    queryFn: fetchAgentRegistry,
    // The registry is rewritten on every `run-services-on-localnet.sh --force`,
    // so a long stale time is fine — refetch on remount is enough.
    staleTime: Number.POSITIVE_INFINITY,
  });

  const byPubkey = useMemo(() => {
    const map = new Map<Address, AgentRegistryEntry>();
    for (const entry of query.data ?? []) {
      map.set(entry.pubkey, entry);
    }
    return map;
  }, [query.data]);

  return { entries: query.data ?? [], byPubkey };
}
