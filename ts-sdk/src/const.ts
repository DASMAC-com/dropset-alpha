import { getMarketSeatDecoder, getSectorDecoder } from "@/generated";

export const LOCALNET_URL = "http://localhost:8899";
export const NIL = 0xffffffff;
export const SECTOR_SIZE = getSectorDecoder().fixedSize;
export const PAYLOAD_SIZE = getMarketSeatDecoder().fixedSize;
export const MARKET_SEED_STR = "market";
