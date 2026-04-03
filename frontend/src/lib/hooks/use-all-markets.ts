"use client";

import { useQuery } from "@tanstack/react-query";
import { fetchAllMarkets } from "@/lib/queries/fetch-all-markets";
import { useRpcClient } from "./use-rpc-client";

export function useAllMarkets() {
  const rpc = useRpcClient();

  return useQuery({
    queryKey: ["markets"],
    queryFn: () => (rpc ? fetchAllMarkets(rpc) : []),
    enabled: !!rpc,
  });
}
