import assert from "node:assert";
import * as fs from "node:fs";
import * as path from "node:path";
import { describe, expect, test } from "@jest/globals";
import type { Address } from "@solana/kit";
import { parseTransaction } from "../parsed-transaction.js";

const GOLDENS_DIR = path.resolve(
  __dirname,
  "../../../transaction-parser/src/goldens",
);

const LOG_TRUNCATED_SUBSTRING = "Log truncated";

function loadGolden(sig: string, encoding = "json") {
  const filePath = path.join(GOLDENS_DIR, sig, `${encoding}.json`);
  const raw = JSON.parse(fs.readFileSync(filePath, "utf-8"));
  return convertBigInts(raw);
}

function convertBigInts(raw: Record<string, unknown>): Record<string, unknown> {
  raw.slot = BigInt(raw.slot as number);
  if (raw.blockTime != null) raw.blockTime = BigInt(raw.blockTime as number);

  const meta = raw.meta as Record<string, unknown> | null;
  if (meta) {
    meta.fee = BigInt(meta.fee as number);
    meta.preBalances = (meta.preBalances as number[]).map(BigInt);
    meta.postBalances = (meta.postBalances as number[]).map(BigInt);
    if (meta.computeUnitsConsumed != null) {
      meta.computeUnitsConsumed = BigInt(meta.computeUnitsConsumed as number);
    }
  }

  return raw;
}

function listGoldenSignatures(): string[] {
  return fs.readdirSync(GOLDENS_DIR).filter((entry) => {
    const full = path.join(GOLDENS_DIR, entry);
    return fs.statSync(full).isDirectory();
  });
}

describe("all goldens parse without error", () => {
  const sigs = listGoldenSignatures();

  test.each(sigs)("%s", (sig) => {
    const golden = loadGolden(sig);
    const parsed = parseTransaction(golden as never);

    expect(parsed.signature).toBe(
      (golden.transaction as { signatures: string[] }).signatures[0],
    );
    expect(parsed.slot).toBe(golden.slot);
    expect(parsed.accounts.length).toBeGreaterThan(0);
    expect(parsed.instructions.length).toBeGreaterThan(0);
  });
});

describe("parse_truncated_logs", () => {
  const SIG =
    "3apVSExwHE5PuoMGHdpBZWbjV79bhcjP2cUTHGwysCKjhBfFcRs2JLDnjpxc6jNhsLu7bNCScNoP2mzrv9dBKCYA";

  test("both truncated and full-logs variants parse and resolve balances", () => {
    const truncated = loadGolden(SIG, "json");
    const fullLogs = loadGolden(SIG, "json_full_logs");

    const truncatedLogs = (truncated.meta as Record<string, unknown>)
      .logMessages as string[];
    const fullLogMessages = (fullLogs.meta as Record<string, unknown>)
      .logMessages as string[];

    expect(truncatedLogs.some((l) => l.includes(LOG_TRUNCATED_SUBSTRING))).toBe(
      true,
    );
    expect(
      fullLogMessages.some((l) => l.includes(LOG_TRUNCATED_SUBSTRING)),
    ).toBe(false);

    const parsedTruncated = parseTransaction(truncated as never);
    const parsedFull = parseTransaction(fullLogs as never);

    expect(parsedTruncated.balances.preLamportBalances.size).toBeGreaterThan(0);
    expect(parsedFull.balances.preLamportBalances.size).toBeGreaterThan(0);
  });
});

describe("parse_correct_balances", () => {
  const SIG =
    "5Vt3URq3RfWdPQkiJEWxDMcCQ65UeRzxoBwCd3vBvwsN54HvEu6s71zXRw5p3VJwfKKiPdmgG7T2NuJT1t3h3QcN";

  test("lamport balances match expected values", () => {
    const golden = loadGolden(SIG);
    const parsed = parseTransaction(golden as never);

    const user = "11113MwGAy1Aq8qkfPuukq892Zn3tV6uGHWoRYLaUBS" as Address;
    expect(parsed.balances.preLamportBalances.get(user)).toBe(179832313698n);
    expect(parsed.balances.postLamportBalances.get(user)).toBe(179832303696n);
  });

  test("token balances match expected values", () => {
    const golden = loadGolden(SIG);
    const parsed = parseTransaction(golden as never);

    // The Rust test derives the ATA address from user + mint. We verify the same
    // balance values exist by finding the token account at the expected index.
    const meta = golden.meta as Record<string, unknown>;
    const preTokenBalances = meta.preTokenBalances as {
      accountIndex: number;
    }[];
    const postTokenBalances = meta.postTokenBalances as {
      accountIndex: number;
    }[];

    // First pre-token balance entry should map to an account with balance 0.
    const preAccount =
      parsed.accounts[preTokenBalances[0].accountIndex].address;
    expect(parsed.balances.preTokenBalances.get(preAccount)).toBe(0n);

    // First post-token balance entry should map to an account with balance 10000.
    const postAccount =
      parsed.accounts[postTokenBalances[0].accountIndex].address;
    expect(parsed.balances.postTokenBalances.get(postAccount)).toBe(10000n);
  });
});

describe("parse_balances_with_loaded_addresses", () => {
  const SIG =
    "3apVSExwHE5PuoMGHdpBZWbjV79bhcjP2cUTHGwysCKjhBfFcRs2JLDnjpxc6jNhsLu7bNCScNoP2mzrv9dBKCYA";

  test("ALT-loaded addresses resolve and all token accounts are in balance maps", () => {
    const golden = loadGolden(SIG, "json_full_logs");
    const parsed = parseTransaction(golden as never);

    const meta = golden.meta as Record<string, unknown>;
    const loadedAddresses = meta.loadedAddresses as {
      writable: string[];
      readonly: string[];
    };
    expect(
      loadedAddresses.writable.length + loadedAddresses.readonly.length,
    ).toBeGreaterThan(0);

    const preTokenBalances = meta.preTokenBalances as {
      accountIndex: number;
    }[];
    const postTokenBalances = meta.postTokenBalances as {
      accountIndex: number;
    }[];

    for (const tokenBalances of [preTokenBalances, postTokenBalances]) {
      for (const b of tokenBalances) {
        const tokenAccount = parsed.accounts[b.accountIndex].address;
        const inPre = parsed.balances.preTokenBalances.has(tokenAccount);
        const inPost = parsed.balances.postTokenBalances.has(tokenAccount);
        assert.ok(inPre || inPost, `${tokenAccount} not in any balance map`);
      }
    }
  });
});
