import { describe, it } from "@jest/globals";
import { getDropsetMarkets, getRpcClient } from "@/utils";

describe("Dropset market accounts", () => {
  it("should get all dropset market accounts", async () => {
    const rpcClient = getRpcClient();
    const markets = await getDropsetMarkets(rpcClient);

    console.log(markets);
  });
});
