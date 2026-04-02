export const LOCALNET_URL = "http://localhost:8899";

import { getMarketSeatDecoder, getSectorDecoder } from "@/generated";

export const NIL = 0xffffffff;
export const SECTOR_SIZE = getSectorDecoder().fixedSize;
export const PAYLOAD_SIZE = getMarketSeatDecoder().fixedSize;
