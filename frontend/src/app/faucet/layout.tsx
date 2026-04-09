import { FAUCET_URL } from "@/lib/env";
import { FaucetProvider } from "./FaucetProvider";

async function getFaucetInfo() {
  const res = await fetch(`${FAUCET_URL}/info`, { cache: "no-store" });
  if (!res.ok) throw new Error("Failed to fetch faucet info");
  return res.json();
}

export default async function FaucetLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const info = await getFaucetInfo();
  return <FaucetProvider info={info}>{children}</FaucetProvider>;
}
