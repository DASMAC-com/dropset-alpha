"use client";

import type { Address } from "@solana/addresses";
import { useQuery } from "@tanstack/react-query";
import { fetchMarket } from "@/lib/queries/fetch-market";

export function useMarket(address: Address) {
  return useQuery({
    queryKey: ["market", address],
    queryFn: () => fetchMarket(address),
    enabled: !!address,
  });
}
