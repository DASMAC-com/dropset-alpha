"use client";

import { useEffect, useState } from "react";
import { getRpcFromEnv } from "@/lib/env";
import type { RpcClient } from "@/ts-sdk";

export function useRpcClient(): RpcClient | undefined {
  const [rpc, setRpc] = useState<RpcClient | undefined>(undefined);

  useEffect(() => {
    setRpc(getRpcFromEnv());
  }, []);

  return rpc;
}
