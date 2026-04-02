import { describe, expect, it } from "@jest/globals";
import { toMarketViewAll } from "@/dropset-interface";
import { deriveMarketAddress, getDropsetMarkets, getRpcClient } from "@/utils";

describe("Dropset market accounts", () => {
  it("should decode all dropset market accounts", async () => {
    const rpcClient = getRpcClient();
    const markets = await getDropsetMarkets(rpcClient);
    const res = markets.map(
      ([address, market]) => [address, toMarketViewAll(market)] as const,
    );

    for (const [address, view] of res) {
      const [derivedMarketAddress, _] = await deriveMarketAddress(
        view.header.baseMint,
        view.header.quoteMint,
      );
      expect(derivedMarketAddress).toBe(address);
    }
  });
});
