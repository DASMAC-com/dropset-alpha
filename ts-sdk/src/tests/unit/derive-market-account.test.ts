import { describe, expect, it } from "@jest/globals";
import type { Address } from "@solana/kit";
import { DROPSET_PROGRAM_ADDRESS } from "@/generated";
import { deriveMarketAddress } from "@/utils";

describe("Dropset market account derivation", () => {
  it("should derive a dropset market address correctly", async () => {
    const [implicitDerivedAddress, implicitBump] = await deriveMarketAddress(
      "Fek6Mh9MFYQMfpEiZtJ7TB81EKpxkB4mDvKotXQkmfp9" as Address<string>,
      "8NQESijBmKikaFiHLTnMj2RSZ3jDwAfiKuZTN5Wn6iDx" as Address<string>,
    );
    const [explicitDerivedAddress, explicitBump] = await deriveMarketAddress(
      "Fek6Mh9MFYQMfpEiZtJ7TB81EKpxkB4mDvKotXQkmfp9" as Address<string>,
      "8NQESijBmKikaFiHLTnMj2RSZ3jDwAfiKuZTN5Wn6iDx" as Address<string>,
      DROPSET_PROGRAM_ADDRESS,
    );

    const expectedBump = 247;
    const expectedAddress = "2jxZxWuK6X9banZdkAHVki8v3VCMdEV2rBcCJKBovr5V";

    expect(implicitDerivedAddress).toEqual(expectedAddress);
    expect(explicitDerivedAddress).toEqual(expectedAddress);
    expect(implicitBump).toEqual(expectedBump);
    expect(explicitBump).toEqual(expectedBump);
  });
});
