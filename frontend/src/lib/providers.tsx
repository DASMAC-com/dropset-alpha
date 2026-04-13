"use client";

import { autoDiscover, createClient } from "@solana/client";
import { SolanaProvider } from "@solana/react-hooks";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import dynamic from "next/dynamic";
import { type ReactNode, useState } from "react";
import Toaster from "@/components/Toaster";
import { PUBLIC_RPC_URL, PUBLIC_WS_URL } from "./env";

const ReactQueryDevtools = dynamic(
  () =>
    import("@tanstack/react-query-devtools").then(
      (mod) => mod.ReactQueryDevtools,
    ),
  { ssr: false },
);

const client = createClient({
  endpoint: PUBLIC_RPC_URL,
  websocketEndpoint: PUBLIC_WS_URL,
  walletConnectors: autoDiscover(),
});

export function Providers({ children }: { children: ReactNode }) {
  const isDevelopment = process.env.NODE_ENV === "development";

  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 30_000,
            refetchOnWindowFocus: false,
          },
        },
      }),
  );

  return (
    <SolanaProvider client={client}>
      <QueryClientProvider client={queryClient}>
        {children}
        {isDevelopment ? <ReactQueryDevtools initialIsOpen={false} /> : null}
        <Toaster />
      </QueryClientProvider>
    </SolanaProvider>
  );
}
