import { NextResponse } from "next/server";
import { FAUCET_URL } from "@/lib/env";

/**
 * Forward proxy to the Rust faucet service.
 *
 * See the upstream handler at
 * {@link [faucet/src/main.rs](../../../../services/faucet/src/main.rs)}.
 */
export async function POST(request: Request) {
  try {
    const body = await request.json();

    const res = await fetch(`${FAUCET_URL}/faucet`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });

    const data = await res.json();

    return NextResponse.json(data, { status: res.status });
  } catch {
    return NextResponse.json({ error: "Faucet unreachable" }, { status: 502 });
  }
}
