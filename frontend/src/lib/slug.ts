/**
 * Compute the shortest unique prefix for each address in the set.
 * Returns a map from full address → shortest unambiguous prefix.
 * Minimum prefix length is 6 characters.
 */
export function buildPrefixMap(addresses: string[]): Map<string, string> {
  const MIN_LEN = 6;
  const map = new Map<string, string>();

  for (const addr of addresses) {
    let len = MIN_LEN;
    while (len < addr.length) {
      const prefix = addr.slice(0, len);
      const collisions = addresses.filter((a) => a.startsWith(prefix));
      if (collisions.length === 1) break;
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
export function resolveSlug(slug: string, addresses: string[]): string | null {
  // Exact match first
  if (addresses.includes(slug)) return slug;

  // Prefix match
  const matches = addresses.filter((a) => a.startsWith(slug));
  if (matches.length === 1) return matches[0];

  return null;
}
