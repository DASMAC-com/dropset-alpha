"use client";

import { useEffect, useState } from "react";
import { getRpcClient, type RpcClient } from "@/ts-sdk";

export function useRpcClient(): RpcClient | undefined {
  const [rpc, setRpc] = useState<RpcClient | undefined>(undefined);

  useEffect(() => {
    setRpc(getRpcClient());
  }, []);

  return rpc;
}
