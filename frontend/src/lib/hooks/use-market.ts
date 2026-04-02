"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchMarket } from "@/lib/queries/fetch-market";

export function useMarket(address: string) {
  return useQuery({
    queryKey: ["market", address],
    queryFn: () => fetchMarket(address),
    enabled: !!address,
  });
}
