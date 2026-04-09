import { FAUCET_URL } from "@/lib/env";

/** Forward proxy to the Rust faucet /info endpoint. */
export async function GET() {
  const res = await fetch(`${FAUCET_URL}/info`);
  const data = await res.json();
  return Response.json(data, { status: res.status });
}
