import { Decimal } from "decimal.js";

export function truncateAddress(addr: string): string {
  return `${addr.slice(0, 4)}…${addr.slice(-4)}`;
}

export function formatBalance(uiAmount: string): string {
  const d = new Decimal(uiAmount);
  if (d.isZero()) return "0";
  return d.toSignificantDigits(3).toString();
}

export function sanitizeDecimalInput(value: string): string {
  let result = "";
  let hasDot = false;
  for (const ch of value) {
    if (ch >= "0" && ch <= "9") {
      result += ch;
    } else if (ch === "." && !hasDot) {
      hasDot = true;
      result += ch;
    }
  }
  return result;
}

export function uiToAtoms(uiAmount: string, decimals: number): bigint {
  if (!uiAmount || uiAmount === ".") return 0n;
  const d = new Decimal(uiAmount).mul(Decimal.pow(10, decimals));
  return BigInt(d.toFixed(0));
}
