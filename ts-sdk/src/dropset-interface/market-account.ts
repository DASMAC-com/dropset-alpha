import type {
  Address,
  FixedSizeDecoder,
  ReadonlyUint8Array,
} from "@solana/kit";
import { NIL, SECTOR_SIZE } from "@/const";
import type {
  MarketAccount,
  MarketHeader,
  MarketSeat,
  Order,
  Sector,
} from "@/generated";
import {
  getMarketSeatDecoder,
  getOrderDecoder,
  getSectorDecoder,
} from "@/generated";
import type { Flatten, SectorIndex } from "@/types";

export type MarketViewAll = {
  header: MarketHeader;
  seats: MarketSeatView[];
  bids: OrderView[];
  asks: OrderView[];
  users: Map<Address, MarketUserData>;
};

export type MarketSeatView = Flatten<
  {
    prevIndex: SectorIndex;
    index: SectorIndex;
    nextIndex: SectorIndex;
  } & MarketSeat
>;

export type OrderView = Flatten<
  {
    prevIndex: SectorIndex;
    index: SectorIndex;
    nextIndex: SectorIndex;
  } & Omit<Order, "padding">
>;

export type MarketUserData = {
  seat: MarketSeatView;
  bids: OrderView[];
  asks: OrderView[];
};

/**
 * Collects {@link Sector}s from a {@link MarketAccount}.
 *
 * Similar to the Rust `LinkedListIter` which traverses sectors via
 * the `next` pointer in each sector header.
 */
export function collectSectors(
  sectorsBytes: ReadonlyUint8Array,
  head: SectorIndex,
): [SectorIndex, Sector][] {
  const decoder = getSectorDecoder();
  const result: [SectorIndex, Sector][] = [];
  let curr = head;
  let iters = 0;

  if (sectorsBytes.length % SECTOR_SIZE !== 0) {
    const msg = `Malformed sectors bytes, ${sectorsBytes.length} is not divisible by ${SECTOR_SIZE}`;
    throw new Error(msg);
  }
  const maxNumSectors = sectorsBytes.length / SECTOR_SIZE;

  while (curr !== NIL) {
    const offset = curr * SECTOR_SIZE;

    const bytesAfterOffset = sectorsBytes.length - offset;
    if (bytesAfterOffset < SECTOR_SIZE) {
      const msg = `Bytes after offset (${bytesAfterOffset}) is < \`SECTOR_SIZE\` (${SECTOR_SIZE})`;
      throw new Error(msg);
    }

    const sector = decoder.decode(sectorsBytes, offset);
    result.push([curr, sector]);
    curr = sector.next;

    if (iters >= maxNumSectors) {
      const msg = `Sector iterator ran more than the max # of sectors (${maxNumSectors}) times`;
      throw new Error(msg);
    }
    iters += 1;
  }

  return result;
}

type SectorPayload = MarketSeat | Order;

/** Decodes a sector's payload (number[]) as a typed value. */
function decodePayload<T extends SectorPayload>(
  payload: Array<number>,
  decoder: FixedSizeDecoder<T>,
): T {
  return decoder.decode(new Uint8Array(payload));
}

/** Sector metadata included with every decoded payload. */
type WithSectorMeta<T> = T & {
  prevIndex: SectorIndex;
  index: SectorIndex;
  nextIndex: SectorIndex;
};

/** Collects sectors from a linked list, decoding each payload with sector metadata. */
function collectFromDll<T extends SectorPayload>(
  sectorsBytes: ReadonlyUint8Array,
  head: SectorIndex,
  decoder: FixedSizeDecoder<T>,
): WithSectorMeta<T>[] {
  return collectSectors(sectorsBytes, head).map(([index, sector]) => ({
    prevIndex: sector.prev,
    index,
    nextIndex: sector.next,
    ...decodePayload(sector.payload, decoder),
  }));
}

function collectSeats(market: MarketAccount): MarketSeatView[] {
  return collectFromDll(
    market.sectors,
    market.header.seatsDllHead,
    getMarketSeatDecoder(),
  );
}

function collectOrders(market: MarketAccount, head: SectorIndex): OrderView[] {
  return collectFromDll(market.sectors, head, getOrderDecoder()).map(
    ({ padding: _, ...rest }) => rest,
  );
}

export function toMarketViewAll(market: MarketAccount): MarketViewAll {
  const seats = collectSeats(market);
  const bids = collectOrders(market, market.header.bidsDllHead);
  const asks = collectOrders(market, market.header.asksDllHead);

  const seatIndexToUser = new Map(seats.map((v) => [v.index, v.user]));

  // Instantiate all user data with their corresponding seat and empty bids/asks.
  const users = new Map(
    seats.map((seat) => {
      const newUserData: MarketUserData = {
        seat,
        bids: [],
        asks: [],
      };

      return [seat.user, newUserData];
    }),
  );

  // Update each user entry's bids.
  for (const bid of bids) {
    const user = seatIndexToUser.get(bid.userSeatIndex);
    if (typeof user === "undefined") {
      throw new Error("Should find user in seat index map");
    }
    const userData = users.get(user);
    if (typeof userData === "undefined") {
      throw new Error("Should find user in user map");
    }

    userData.bids.push(bid);
  }

  // Update each user entry's asks.
  for (const ask of asks) {
    const user = seatIndexToUser.get(ask.userSeatIndex);
    if (typeof user === "undefined") {
      throw new Error("Should find user in seat index map");
    }
    const userData = users.get(user);
    if (typeof userData === "undefined") {
      throw new Error("Should find user in user map");
    }

    userData.asks.push(ask);
  }

  return {
    header: market.header,
    seats,
    bids,
    asks,
    users,
  };
}
