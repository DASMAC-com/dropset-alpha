import { describe, expect, it } from "@jest/globals";
import {
  deriveMarketAddress,
  fetchDropsetMarketAccounts,
  fetchDropsetMarketViews,
  getRpcClient,
} from "@/ts-sdk/utils";

describe("Dropset market accounts", () => {
  it("should decode all dropset market accounts", async () => {
    const rpcClient = getRpcClient();
    const markets = await fetchDropsetMarketAccounts(rpcClient);

    for (const market of markets) {
      const [derivedMarketAddress, _] = await deriveMarketAddress(
        market.header.baseMint,
        market.header.quoteMint,
      );
      expect(derivedMarketAddress).toBe(market.address);
    }
  });

  it("should decode all dropset market accounts into market views", async () => {
    const rpcClient = getRpcClient();
    const markets = await fetchDropsetMarketViews(rpcClient);

    for (const market of markets) {
      const [derivedMarketAddress, _] = await deriveMarketAddress(
        market.header.baseMint,
        market.header.quoteMint,
      );
      expect(derivedMarketAddress).toBe(market.address);
    }
  });
});
