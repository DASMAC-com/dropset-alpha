"use client";

import * as Dialog from "@radix-ui/react-dialog";
import * as Popover from "@radix-ui/react-popover";
import type { Address } from "@solana/addresses";
import { useWalletModalState } from "@solana/react-hooks";
import { Check, Copy } from "lucide-react";
import Image from "next/image";
import { useCallback, useState } from "react";
import { copyAddressHelper } from "@/lib/utils";

function CopyAddress({ address }: { address: Address }) {
  const [recentlyCopied, setRecentlyCopied] = useState(false);
  const short = `${address.slice(0, 8)}...${address.slice(-8)}`;

  return (
    <button
      type="button"
      onClick={async () => {
        const successful = await copyAddressHelper(address);
        if (successful) {
          setRecentlyCopied(true);
          setTimeout(() => setRecentlyCopied(false), 1500);
        }
      }}
      className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left transition-colors hover:bg-muted"
    >
      <span className="flex-1 font-mono text-muted-fg text-xs">{short}</span>
      {recentlyCopied ? (
        <Check className="h-3.5 w-3.5 text-green-500" />
      ) : (
        <Copy className="h-3.5 w-3.5 text-muted-fg" />
      )}
    </button>
  );
}

export default function WalletConnector() {
  const modal = useWalletModalState({ closeOnConnect: true });

  const handleConnect = useCallback(
    (connectorId: string) => {
      modal.connect(connectorId);
    },
    [modal.connect],
  );

  if (!modal.isReady) {
    return (
      <div className="ml-auto h-9 w-32 animate-pulse rounded-lg bg-muted" />
    );
  }

  // Connected — popover for disconnect
  if (modal.status === "connected" && modal.wallet) {
    const addr = modal.wallet.account.address;
    const short = `${addr.slice(0, 4)}...${addr.slice(-4)}`;
    const icon = modal.wallet.connector.icon;

    return (
      <Popover.Root>
        <Popover.Trigger asChild>
          <button
            type="button"
            className="ml-auto flex items-center gap-2 rounded-lg border border-border bg-muted px-3 py-1.5 font-medium text-foreground text-sm transition-colors hover:bg-border"
          >
            {icon && (
              <Image
                src={icon}
                alt=""
                width={16}
                height={16}
                className="h-4 w-4 rounded-sm"
                unoptimized
              />
            )}
            <span className="font-mono tabular-nums">{short}</span>
          </button>
        </Popover.Trigger>
        <Popover.Portal>
          <Popover.Content
            align="end"
            sideOffset={8}
            className="z-50 w-56 rounded-xl border border-border bg-background p-2 shadow-lg"
          >
            <CopyAddress address={addr} />
            <button
              type="button"
              onClick={() => modal.disconnect()}
              className="w-full rounded-lg px-3 py-2 text-left text-red-500 text-sm transition-colors hover:bg-muted"
            >
              Disconnect
            </button>
          </Popover.Content>
        </Popover.Portal>
      </Popover.Root>
    );
  }

  // Disconnected — centered modal for wallet selection
  return (
    <Dialog.Root
      open={modal.isOpen}
      onOpenChange={(open) => (open ? modal.open() : modal.close())}
    >
      <Dialog.Trigger asChild>
        <button
          type="button"
          className="ml-auto rounded-lg bg-accent px-4 py-1.5 font-medium text-sm text-white transition-colors hover:bg-accent-hover"
        >
          Connect Wallet
        </button>
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 z-50 w-80 -translate-x-1/2 -translate-y-1/2 rounded-2xl border border-border bg-background shadow-xl">
          <div className="flex items-center justify-between border-border border-b px-5 py-4">
            <Dialog.Title className="font-semibold text-foreground">
              Connect a wallet
            </Dialog.Title>
            <Dialog.Close className="rounded-md p-1 text-muted-fg transition-colors hover:bg-muted hover:text-foreground">
              <svg
                width="14"
                height="14"
                viewBox="0 0 14 14"
                fill="none"
                className="stroke-current"
                strokeWidth="2"
                strokeLinecap="round"
              >
                <title>Close</title>
                <path d="M1 1l12 12M13 1L1 13" />
              </svg>
            </Dialog.Close>
          </div>

          <div className="p-3">
            {modal.connectors.length === 0 && (
              <p className="px-3 py-6 text-center text-muted-fg text-sm">
                No wallets detected.
              </p>
            )}

            {modal.connectors.map((connector) => (
              <button
                key={connector.id}
                type="button"
                disabled={modal.status === "connecting"}
                onClick={() => handleConnect(connector.id)}
                className="flex w-full items-center gap-3 rounded-lg px-3 py-3 text-left text-sm transition-colors hover:bg-muted disabled:opacity-50"
              >
                {connector.icon ? (
                  <Image
                    src={connector.icon}
                    alt=""
                    width={32}
                    height={32}
                    className="h-8 w-8 rounded-lg"
                    unoptimized
                  />
                ) : (
                  <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted font-bold text-muted-fg text-xs">
                    {connector.name.charAt(0)}
                  </div>
                )}

                <span className="flex-1 font-medium text-foreground">
                  {connector.name}
                </span>

                {connector.ready && (
                  <span className="text-accent text-xs">Detected</span>
                )}
              </button>
            ))}

            {modal.status === "connecting" && (
              <div className="px-3 py-3 text-center text-muted-fg text-xs">
                Connecting...
              </div>
            )}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
