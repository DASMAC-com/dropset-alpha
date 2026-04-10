import { FAUCET_URL } from "@/lib/env";
import { FaucetProvider } from "./FaucetProvider";

async function getFaucetInfo() {
  try {
    const res = await fetch(`${FAUCET_URL}/info`, { cache: "no-store" });
    if (!res.ok) return null;
    return res.json();
  } catch {
    return null;
  }
}

export default async function FaucetLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const info = await getFaucetInfo();
  if (!info) {
    return (
      <div className="mx-auto max-w-lg px-6 py-8">
        <h1 className="font-semibold text-2xl tracking-tight">Faucet</h1>
        <p className="mt-4 text-muted-fg">Faucet service is unavailable.</p>
      </div>
    );
  }
  return <FaucetProvider info={info}>{children}</FaucetProvider>;
}
