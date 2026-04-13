import { NextResponse } from "next/server";
import { FAUCET_URL } from "@/lib/env";

/** Forward proxy to the Rust faucet /info endpoint. */
export async function GET() {
  try {
    const res = await fetch(`${FAUCET_URL}/info`);
    const data = await res.json();
    return NextResponse.json(data, { status: res.status });
  } catch {
    return NextResponse.json({ error: "Faucet unreachable" }, { status: 502 });
  }
}
