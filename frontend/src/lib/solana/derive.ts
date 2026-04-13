import type { Address } from "@solana/addresses";
import { getAddressEncoder, getProgramDerivedAddress } from "@solana/kit";
import { ASSOCIATED_TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";

export async function deriveAta(
  owner: Address,
  mint: Address,
  tokenProgram: Address,
): Promise<Address> {
  const encoder = getAddressEncoder();
  const [ata] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    seeds: [
      encoder.encode(owner),
      encoder.encode(tokenProgram),
      encoder.encode(mint),
    ],
  });
  return ata;
}
