"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchAllMarkets } from "@/lib/queries/fetch-all-markets";
import { rpcClient } from "../rpc";

export function useAllMarkets() {
  return useQuery({
    queryKey: ["markets"],
    queryFn: () => fetchAllMarkets(rpcClient),
  });
}
