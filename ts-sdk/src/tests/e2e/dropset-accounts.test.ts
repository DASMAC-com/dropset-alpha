import { describe, expect, it } from "@jest/globals";
import {
  deriveMarketAddress,
  fetchDropsetMarketAccount,
  fetchDropsetMarketAccounts,
  fetchDropsetMarketView,
  fetchDropsetMarketViews,
  getRpcClient,
} from "@/ts-sdk/utils";
import { DROPSET_PROGRAM_ADDRESS } from "@/ts-sdk/generated/programs/dropset";
import assert from "node:assert";
import type { DropsetMarketAccount, DropsetMarketView } from "@/ts-sdk/types";

const deriveCheck = async (
  market: DropsetMarketAccount | DropsetMarketView,
) => {
  const [derivedMarketAddress, _] = await deriveMarketAddress(
    market.header.baseMint,
    market.header.quoteMint,
  );
  expect(derivedMarketAddress).toBe(market.address);
};

describe("Dropset market accounts", () => {
  it("should decode all dropset market accounts", async () => {
    const rpcClient = getRpcClient();
    const markets = await fetchDropsetMarketAccounts(rpcClient);
    expect(markets.length).toBeGreaterThanOrEqual(1);

    for (const market of markets) {
      await deriveCheck(market);
    }
  });

  it("should decode all dropset market accounts into market views", async () => {
    const rpcClient = getRpcClient();
    const markets = await fetchDropsetMarketViews(rpcClient);
    expect(markets.length).toBeGreaterThanOrEqual(1);

    for (const market of markets) {
      await deriveCheck(market);
    }
  });

  it("should decode one dropset market account", async () => {
    const rpcClient = getRpcClient();
    const marketAddresses = await rpcClient
      .getProgramAccounts(DROPSET_PROGRAM_ADDRESS, { encoding: "base64" })
      .send()
      .then((markets) => markets.map((m) => m.pubkey));

    expect(marketAddresses.length).toBeGreaterThanOrEqual(1);
    const first = marketAddresses.at(0);
    assert(first !== undefined);
    const market = await fetchDropsetMarketAccount(rpcClient, first);
    assert(market !== undefined);
    await deriveCheck(market);
  });

  it("should decode one dropset market view", async () => {
    const rpcClient = getRpcClient();
    const marketAddresses = await rpcClient
      .getProgramAccounts(DROPSET_PROGRAM_ADDRESS, { encoding: "base64" })
      .send()
      .then((markets) => markets.map((m) => m.pubkey));

    expect(marketAddresses.length).toBeGreaterThanOrEqual(1);
    const first = marketAddresses.at(0);
    assert(first !== undefined);
    const marketView = await fetchDropsetMarketView(rpcClient, first);
    assert(marketView !== undefined);
    await deriveCheck(marketView);
  });
});
