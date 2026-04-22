"use client";

import { useQuery } from "@tanstack/react-query";
import type { Participant, ParticipantRole } from "@/app/api/participants/route";

export type { Participant, ParticipantRole };

export type ParticipantLike = {
  address: string;
  label: string;
  role: ParticipantRole | "you" | "unknown";
};

const ROLE_STYLE: Record<ParticipantLike["role"], string> = {
  retail: "bg-sky-500/15 text-sky-400 border-sky-500/30",
  whale: "bg-amber-500/15 text-amber-400 border-amber-500/30",
  sniper: "bg-violet-500/15 text-violet-400 border-violet-500/30",
  noise: "bg-slate-500/15 text-slate-300 border-slate-500/30",
  passive: "bg-emerald-500/15 text-emerald-400 border-emerald-500/30",
  aggressive: "bg-rose-500/15 text-rose-400 border-rose-500/30",
  maker: "bg-fuchsia-500/15 text-fuchsia-400 border-fuchsia-500/30",
  faucet: "bg-teal-500/15 text-teal-400 border-teal-500/30",
  you: "bg-primary/20 text-primary border-primary/40",
  unknown: "bg-muted/40 text-muted-fg border-border",
};

export function roleStyle(role: ParticipantLike["role"]): string {
  return ROLE_STYLE[role];
}

export const ALL_ROLES: ParticipantLike["role"][] = [
  "retail",
  "whale",
  "sniper",
  "noise",
  "passive",
  "aggressive",
  "maker",
  "faucet",
  "you",
  "unknown",
];

type ParticipantsResponse = {
  participants: Participant[];
  byAddress: Record<string, Participant>;
};

async function fetchParticipants(): Promise<ParticipantsResponse> {
  const res = await fetch("/api/participants");
  if (!res.ok) throw new Error(`Failed: ${res.status}`);
  return res.json();
}

/**
 * Addresses are fresh per localnet run, but stable for the whole session.
 * Fetch once and cache aggressively.
 */
export function useParticipants() {
  return useQuery({
    queryKey: ["participants"],
    queryFn: fetchParticipants,
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
    retry: 1,
  });
}

export function resolveParticipant(
  address: string,
  byAddress: Record<string, Participant> | undefined,
  walletAddress: string | undefined,
): ParticipantLike {
  if (walletAddress && address === walletAddress) {
    return { address, label: "you", role: "you" };
  }
  const hit = byAddress?.[address];
  if (hit) return hit;
  return { address, label: address, role: "unknown" };
}
