"use client";

import type { Address } from "@solana/addresses";
import { useQuery } from "@tanstack/react-query";
import { fetchMarket } from "@/lib/queries/fetch-market";
import { rpcClient } from "../rpc";

export function useMarket(address: Address) {
  return useQuery({
    queryKey: ["market", address],
    queryFn: () => fetchMarket(address, rpcClient),
    enabled: !!address,
  });
}
