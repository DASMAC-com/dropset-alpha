import { describe, it } from "@jest/globals";
import { toMarketViewAll } from "@/dropset-interface";
import { getDropsetMarkets, getRpcClient } from "@/utils";

describe("Dropset market accounts", () => {
  it("should decode all dropset market accounts", async () => {
    const rpcClient = getRpcClient();
    const markets = await getDropsetMarkets(rpcClient);
    const marketViews = markets.map(([_, mkt]) => toMarketViewAll(mkt));

    console.dir(marketViews, { depth: null });
  });
});
