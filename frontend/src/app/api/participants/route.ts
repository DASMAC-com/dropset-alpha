import { promises as fs } from "node:fs";
import path from "node:path";
import { createKeyPairFromBytes, getAddressFromPublicKey } from "@solana/kit";
import { NextResponse } from "next/server";

export const dynamic = "force-dynamic";

export type ParticipantRole =
  | "retail"
  | "whale"
  | "sniper"
  | "noise"
  | "passive"
  | "aggressive"
  | "maker"
  | "faucet";

export type Participant = {
  address: string;
  label: string;
  role: ParticipantRole;
};

type KeypairSource = {
  file: string;
  label: string;
  role: ParticipantRole;
};

const SERVICES_DIR = path.join(process.cwd(), "..", "services");

// The taker-bot config.toml lists one `[[agent]]` per archetype; the role for
// each is encoded in the keypair filename (`<archetype>-<n>.json`).
async function takerSources(): Promise<KeypairSource[]> {
  const dir = path.join(SERVICES_DIR, "taker-bot", "keypairs");
  const entries = await fs.readdir(dir).catch(() => [] as string[]);
  return entries
    .filter((f) => f.endsWith(".json"))
    .map((f) => {
      const name = f.replace(/\.json$/, "");
      const archetype = name.split("-")[0] as ParticipantRole;
      return {
        file: path.join(dir, f),
        label: name,
        role: archetype,
      };
    });
}

async function deriveAddress(file: string): Promise<string> {
  const raw = await fs.readFile(file, "utf8");
  const bytes = new Uint8Array(JSON.parse(raw) as number[]);
  const { publicKey } = await createKeyPairFromBytes(bytes);
  return (await getAddressFromPublicKey(publicKey)) as string;
}

export async function GET() {
  try {
    const sources: KeypairSource[] = [
      ...(await takerSources()),
      {
        file: path.join(SERVICES_DIR, "maker-bot", "keypair.json"),
        label: "maker",
        role: "maker",
      },
      {
        file: path.join(SERVICES_DIR, "faucet", "keypair.json"),
        label: "faucet",
        role: "faucet",
      },
    ];

    const resolved = await Promise.all(
      sources.map(async ({ file, label, role }) => {
        try {
          const address = await deriveAddress(file);
          return { address, label, role } satisfies Participant;
        } catch {
          return null;
        }
      }),
    );

    const participants = resolved.filter((p): p is Participant => p !== null);

    const byAddress: Record<string, Participant> = {};
    for (const p of participants) byAddress[p.address] = p;

    return NextResponse.json({ participants, byAddress });
  } catch (e) {
    const msg = e instanceof Error ? e.message : "Unknown error";
    return NextResponse.json(
      { error: `Failed to read participants: ${msg}` },
      { status: 500 },
    );
  }
}
