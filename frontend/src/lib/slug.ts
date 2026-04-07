import type { Address } from "@solana/addresses";

/**
 * Compute the shortest unique prefix for each address in the set.
 * Returns a map from full address → shortest unambiguous prefix.
 * Minimum prefix length is 6 characters.
 */
export function buildPrefixMap(addresses: Address[]): Map<Address, string> {
  const MIN_LEN = 6;
  const sorted = [...addresses].sort();
  const map = new Map<Address, string>();

  for (let i = 0; i < sorted.length; i++) {
    const addr = sorted[i];
    const prev = sorted[i - 1] ?? "";
    const next = sorted[i + 1] ?? "";

    // Find the length needed to distinguish from both neighbors.
    let len = MIN_LEN;
    while (
      len < addr.length &&
      (addr.slice(0, len) === prev.slice(0, len) ||
        addr.slice(0, len) === next.slice(0, len))
    ) {
      len++;
    }
    map.set(addr, addr.slice(0, len));
  }

  return map;
}

/**
 * Resolve a (possibly truncated) slug to a full market address.
 * Returns the full address if exactly one match, or null if ambiguous / none.
 */
export function resolveSlug(
  slug: string,
  addresses: Address[],
): Address | null {
  const matches = addresses.filter((a) => a.startsWith(slug));
  if (matches.length === 1) return matches[0];
  return null;
}
