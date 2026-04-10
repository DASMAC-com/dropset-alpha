import type { ReadonlyUint8Array } from "@solana/kit";
import {
  CANCEL_ORDER_EVENT_DISCRIMINATOR,
  type CancelOrderEventInstructionData,
  CLOSE_SEAT_EVENT_DISCRIMINATOR,
  type CloseSeatEventInstructionData,
  DEPOSIT_EVENT_DISCRIMINATOR,
  type DepositEventInstructionData,
  EXPAND_MARKET_EVENT_DISCRIMINATOR,
  type ExpandMarketEventInstructionData,
  FILL_EVENT_DISCRIMINATOR,
  type FillEventInstructionData,
  getCancelOrderEventInstructionDataDecoder,
  getCloseSeatEventInstructionDataDecoder,
  getDepositEventInstructionDataDecoder,
  getExpandMarketEventInstructionDataDecoder,
  getFillEventInstructionDataDecoder,
  getHeaderEventInstructionDataDecoder,
  getPostOrderEventInstructionDataDecoder,
  getRegisterMarketEventInstructionDataDecoder,
  getWithdrawEventInstructionDataDecoder,
  HEADER_EVENT_DISCRIMINATOR,
  type HeaderEventInstructionData,
  POST_ORDER_EVENT_DISCRIMINATOR,
  type PostOrderEventInstructionData,
  REGISTER_MARKET_EVENT_DISCRIMINATOR,
  type RegisterMarketEventInstructionData,
  WITHDRAW_EVENT_DISCRIMINATOR,
  type WithdrawEventInstructionData,
} from "@/ts-sdk";

const FLUSH_EVENTS_TAG = 8;

export type DropsetEvent =
  | { kind: "header"; data: HeaderEventInstructionData }
  | { kind: "deposit"; data: DepositEventInstructionData }
  | { kind: "withdraw"; data: WithdrawEventInstructionData }
  | { kind: "registerMarket"; data: RegisterMarketEventInstructionData }
  | { kind: "postOrder"; data: PostOrderEventInstructionData }
  | { kind: "cancelOrder"; data: CancelOrderEventInstructionData }
  | { kind: "fill"; data: FillEventInstructionData }
  | { kind: "closeSeat"; data: CloseSeatEventInstructionData }
  | { kind: "expandMarket"; data: ExpandMarketEventInstructionData };

const headerDecoder = getHeaderEventInstructionDataDecoder();
const depositDecoder = getDepositEventInstructionDataDecoder();
const withdrawDecoder = getWithdrawEventInstructionDataDecoder();
const registerMarketDecoder = getRegisterMarketEventInstructionDataDecoder();
const postOrderDecoder = getPostOrderEventInstructionDataDecoder();
const cancelOrderDecoder = getCancelOrderEventInstructionDataDecoder();
const fillDecoder = getFillEventInstructionDataDecoder();
const closeSeatDecoder = getCloseSeatEventInstructionDataDecoder();
const expandMarketDecoder = getExpandMarketEventInstructionDataDecoder();

type EventEntry = {
  discriminator: number;
  kind: DropsetEvent["kind"];
  decoder: {
    fixedSize: number;
    decode: (data: ReadonlyUint8Array, offset: number) => unknown;
  };
};

const EVENT_TABLE: EventEntry[] = [
  {
    discriminator: HEADER_EVENT_DISCRIMINATOR,
    kind: "header",
    decoder: headerDecoder,
  },
  {
    discriminator: DEPOSIT_EVENT_DISCRIMINATOR,
    kind: "deposit",
    decoder: depositDecoder,
  },
  {
    discriminator: WITHDRAW_EVENT_DISCRIMINATOR,
    kind: "withdraw",
    decoder: withdrawDecoder,
  },
  {
    discriminator: REGISTER_MARKET_EVENT_DISCRIMINATOR,
    kind: "registerMarket",
    decoder: registerMarketDecoder,
  },
  {
    discriminator: POST_ORDER_EVENT_DISCRIMINATOR,
    kind: "postOrder",
    decoder: postOrderDecoder,
  },
  {
    discriminator: CANCEL_ORDER_EVENT_DISCRIMINATOR,
    kind: "cancelOrder",
    decoder: cancelOrderDecoder,
  },
  {
    discriminator: FILL_EVENT_DISCRIMINATOR,
    kind: "fill",
    decoder: fillDecoder,
  },
  {
    discriminator: CLOSE_SEAT_EVENT_DISCRIMINATOR,
    kind: "closeSeat",
    decoder: closeSeatDecoder,
  },
  {
    discriminator: EXPAND_MARKET_EVENT_DISCRIMINATOR,
    kind: "expandMarket",
    decoder: expandMarketDecoder,
  },
];

const DISC_TO_ENTRY = new Map(EVENT_TABLE.map((e) => [e.discriminator, e]));

export function parseDropsetEvents(
  instructionData: ReadonlyUint8Array,
): DropsetEvent[] {
  if (instructionData.length === 0) return [];
  if (instructionData[0] !== FLUSH_EVENTS_TAG) return [];

  // Strip the FlushEvents instruction tag.
  const eventData = instructionData.slice(1);

  // First event must be the header.
  const header = headerDecoder.decode(eventData);
  if (header.discriminator !== HEADER_EVENT_DISCRIMINATOR) return [];

  const numEvents = header.emittedCount;
  let cursor = headerDecoder.fixedSize;
  const events: DropsetEvent[] = [];

  for (let i = 0; i < numEvents; i++) {
    const tag = eventData[cursor];
    const entry = DISC_TO_ENTRY.get(tag);
    if (!entry) throw new Error(`Unknown event tag: ${tag}`);

    const decoded = entry.decoder.decode(eventData, cursor);
    events.push({ kind: entry.kind, data: decoded } as DropsetEvent);
    cursor += entry.decoder.fixedSize;
  }

  return events;
}
