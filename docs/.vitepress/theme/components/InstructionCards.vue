<template>
  <div class="iac">
    <div class="iac-filters">
      <button
        v-for="cat in categories"
        :key="cat"
        :class="['iac-filter', { active: activeCategory === cat }]"
        @click="activeCategory = cat"
      >
        {{ cat }}
      </button>
    </div>

    <div class="iac-grid">
      <div
        v-for="ix in filteredInstructions"
        :key="ix.name"
        class="iac-card"
        :class="{ expanded: expanded === ix.name }"
        @click="expanded = expanded === ix.name ? null : ix.name"
      >
        <div class="iac-card-header">
          <div class="iac-card-top">
            <code class="iac-name">{{ ix.name }}</code>
            <span class="iac-cat-tag" :style="{ background: catColor(ix.category) }">
              {{ ix.category }}
            </span>
          </div>
          <p class="iac-summary">{{ ix.summary }}</p>
          <div class="iac-meta-row">
            <span class="iac-cu">
              <span class="iac-cu-label">CU</span>
              {{ ix.cu.toLocaleString() }}
            </span>
            <span class="iac-accounts-count">
              {{ ix.accounts.length }} accounts
            </span>
            <span class="iac-chevron">{{ expanded === ix.name ? '▲' : '▼' }}</span>
          </div>
        </div>

        <div v-if="expanded === ix.name" class="iac-card-body">
          <div class="iac-section">
            <div class="iac-section-label">Accounts</div>
            <div class="iac-accounts">
              <div v-for="acc in ix.accounts" :key="acc.name" class="iac-account">
                <span class="iac-account-name">
                  <code>{{ acc.name }}</code>
                  <span v-if="acc.writable" class="iac-badge write">mut</span>
                  <span v-if="acc.signer" class="iac-badge sign">signer</span>
                  <span v-if="acc.pda" class="iac-badge pda">PDA</span>
                </span>
                <span class="iac-account-desc">{{ acc.desc }}</span>
              </div>
            </div>
          </div>

          <div class="iac-section" v-if="ix.events.length">
            <div class="iac-section-label">Emits</div>
            <div class="iac-events">
              <span v-for="ev in ix.events" :key="ev" class="iac-event">{{ ev }}</span>
            </div>
          </div>

          <div class="iac-section" v-if="ix.notes">
            <div class="iac-section-label">Notes</div>
            <p class="iac-notes">{{ ix.notes }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from "vue";

const expanded = ref(null);
const activeCategory = ref("All");

const categories = ["All", "Market", "Seats", "Balances", "Orders", "Events"];

const catColors = {
  Market:   "#7C3AED",
  Seats:    "#0EA5E9",
  Balances: "#F59E0B",
  Orders:   "#10B981",
  Events:   "#EC4899",
};

function catColor(cat) {
  return catColors[cat] + "22";
}

const instructions = [
  {
    name: "register_market",
    category: "Market",
    summary: "Creates and initializes a new Dropset market. Sets mints, tick size, lot size, and initial order book capacity.",
    cu: 1200,
    accounts: [
      { name: "market",       writable: true,  signer: false, pda: false, desc: "The new market account (initialized)" },
      { name: "base_mint",    writable: false, signer: false, pda: false, desc: "SPL mint for the base token" },
      { name: "quote_mint",   writable: false, signer: false, pda: false, desc: "SPL mint for the quote token" },
      { name: "base_vault",   writable: true,  signer: false, pda: true,  desc: "Program-owned token account for base" },
      { name: "quote_vault",  writable: true,  signer: false, pda: true,  desc: "Program-owned token account for quote" },
      { name: "authority",    writable: true,  signer: true,  pda: false, desc: "Market authority and fee payer" },
    ],
    events: [],
    notes: "Must be called before any other instruction on a market. Lot size and tick size cannot be changed after registration.",
  },
  {
    name: "expand_market",
    category: "Market",
    summary: "Increases the order book capacity of an existing market by allocating additional sector storage.",
    cu: 480,
    accounts: [
      { name: "market",    writable: true,  signer: false, pda: false, desc: "The market to expand (writable)" },
      { name: "authority", writable: true,  signer: true,  pda: false, desc: "Must match the registered market authority" },
    ],
    events: [],
    notes: "Call this when the book is at capacity and post_order returns an insufficient space error.",
  },
  {
    name: "close_seat",
    category: "Seats",
    summary: "Closes a trader's seat on a market and reclaims the rent lamports to the authority.",
    cu: 175,
    accounts: [
      { name: "market",    writable: true,  signer: false, pda: false, desc: "The market the seat belongs to" },
      { name: "seat",      writable: true,  signer: false, pda: true,  desc: "The seat PDA to close (derived: market + trader)" },
      { name: "trader",    writable: false, signer: true,  pda: false, desc: "The trader who owns the seat" },
      { name: "recipient", writable: true,  signer: false, pda: false, desc: "Receives the reclaimed rent lamports" },
    ],
    events: [],
    notes: "The seat must have no resting orders before it can be closed. Cancel all open orders first.",
  },
  {
    name: "deposit",
    category: "Balances",
    summary: "Transfers base or quote tokens from a trader's wallet into the program's custody.",
    cu: 210,
    accounts: [
      { name: "market",          writable: false, signer: false, pda: false, desc: "The market to deposit into" },
      { name: "trader_balance",  writable: true,  signer: false, pda: true,  desc: "Trader's balance PDA (derived: market + trader)" },
      { name: "trader_token_acct", writable: true, signer: false, pda: false, desc: "Trader's SPL token account (source)" },
      { name: "vault",           writable: true,  signer: false, pda: true,  desc: "Program vault for this mint" },
      { name: "trader",          writable: false, signer: true,  pda: false, desc: "Owner of the token account" },
    ],
    events: [],
    notes: "Funds must be deposited before post_order can reserve them for a resting order.",
  },
  {
    name: "withdraw",
    category: "Balances",
    summary: "Transfers settled base or quote tokens from the program back to a trader's wallet.",
    cu: 195,
    accounts: [
      { name: "market",          writable: false, signer: false, pda: false, desc: "The market to withdraw from" },
      { name: "trader_balance",  writable: true,  signer: false, pda: true,  desc: "Trader's balance PDA" },
      { name: "trader_token_acct", writable: true, signer: false, pda: false, desc: "Trader's SPL token account (destination)" },
      { name: "vault",           writable: true,  signer: false, pda: true,  desc: "Program vault for this mint" },
      { name: "trader",          writable: false, signer: true,  pda: false, desc: "Must match balance account owner" },
    ],
    events: [],
    notes: "Only settled (not reserved) funds can be withdrawn. Reserved funds are unlocked when orders are cancelled or filled.",
  },
  {
    name: "post_order",
    category: "Orders",
    summary: "Posts a resting limit order on the bid or ask side of the book at a specified price and size.",
    cu: 461,
    accounts: [
      { name: "market",         writable: true,  signer: false, pda: false, desc: "The market (order book mutated)" },
      { name: "seat",           writable: false, signer: false, pda: true,  desc: "Trader's seat — must exist and be valid" },
      { name: "trader_balance", writable: true,  signer: false, pda: true,  desc: "Funds reserved from available balance" },
      { name: "trader",         writable: false, signer: true,  pda: false, desc: "Must match seat's recorded trader" },
    ],
    events: ["OrderPlaced"],
    notes: "Uses the price/client-helpers encoding — pass priceMantissa, baseScalar, baseExponentBiased, quoteExponentBiased from toOrderInfoArgs().",
  },
  {
    name: "cancel_order",
    category: "Orders",
    summary: "Removes a resting order from the book by order index. Reserved funds return to available balance.",
    cu: 318,
    accounts: [
      { name: "market",         writable: true,  signer: false, pda: false, desc: "The market (order removed from book)" },
      { name: "seat",           writable: false, signer: false, pda: true,  desc: "Trader's seat" },
      { name: "trader_balance", writable: true,  signer: false, pda: true,  desc: "Reserved funds returned here" },
      { name: "trader",         writable: false, signer: true,  pda: false, desc: "Must own the order being cancelled" },
    ],
    events: ["OrderCancelled"],
    notes: "Requires the order's sector index — get this from OrderView.index via toMarketViewAll().",
  },
  {
    name: "batch_replace",
    category: "Orders",
    summary: "Atomically cancels a set of resting orders and posts new ones in a single transaction.",
    cu: 890,
    accounts: [
      { name: "market",         writable: true,  signer: false, pda: false, desc: "The market" },
      { name: "seat",           writable: false, signer: false, pda: true,  desc: "Trader's seat" },
      { name: "trader_balance", writable: true,  signer: false, pda: true,  desc: "Balance updated atomically" },
      { name: "trader",         writable: false, signer: true,  pda: false, desc: "Must own all orders being cancelled" },
    ],
    events: ["OrderCancelled", "OrderPlaced"],
    notes: "The primary instruction for market makers. Eliminates the latency gap between cancel and re-quote that would exist with separate transactions.",
  },
  {
    name: "market_order",
    category: "Orders",
    summary: "Executes a market order — matches immediately against the best resting orders on the opposite side.",
    cu: 520,
    accounts: [
      { name: "market",         writable: true,  signer: false, pda: false, desc: "The market (matched orders removed)" },
      { name: "seat",           writable: false, signer: false, pda: true,  desc: "Taker's seat" },
      { name: "trader_balance", writable: true,  signer: false, pda: true,  desc: "Taker's balance updated on fill" },
      { name: "trader",         writable: false, signer: true,  pda: false, desc: "The taker" },
    ],
    events: ["Fill"],
    notes: "Does not rest on the book. The order is fully or partially filled against resting orders at the best available prices.",
  },
  {
    name: "flush_events",
    category: "Events",
    summary: "Removes processed events from the market's event queue and reclaims space for new events.",
    cu: 290,
    accounts: [
      { name: "market",    writable: true,  signer: false, pda: false, desc: "The market whose event queue is flushed" },
      { name: "authority", writable: false, signer: true,  pda: false, desc: "Any authorized caller (crank or trader)" },
    ],
    events: [],
    notes: "Call this regularly — either from a dedicated crank service or after each trade. If the event queue fills up, new fills cannot be written.",
  },
];

const filteredInstructions = computed(() =>
  activeCategory.value === "All"
    ? instructions
    : instructions.filter((ix) => ix.category === activeCategory.value)
);
</script>

<style scoped>
.iac {
  margin: 1.5rem 0;
}

.iac-filters {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 1rem;
}

.iac-filter {
  padding: 4px 12px;
  border-radius: 20px;
  border: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg);
  color: var(--vp-c-text-2);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.iac-filter.active {
  background: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
  color: #fff;
}

.iac-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
}

.iac-card {
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.iac-card:hover {
  border-color: var(--vp-c-brand-1);
  box-shadow: 0 2px 12px rgba(124, 58, 237, 0.08);
}

.iac-card.expanded {
  border-color: var(--vp-c-brand-1);
  grid-column: span 2;
}

@media (max-width: 640px) {
  .iac-card.expanded { grid-column: span 1; }
}

.iac-card-header {
  padding: 14px 16px;
}

.iac-card-top {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.iac-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--vp-c-text-1);
}

.iac-cat-tag {
  font-size: 11px;
  padding: 2px 7px;
  border-radius: 10px;
  color: var(--vp-c-text-2);
  font-weight: 500;
}

.iac-summary {
  font-size: 13px;
  color: var(--vp-c-text-2);
  line-height: 1.5;
  margin: 0 0 10px 0;
}

.iac-meta-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.iac-cu {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
  font-family: var(--vp-font-family-mono);
  color: var(--vp-c-text-1);
  font-weight: 600;
}

.iac-cu-label {
  font-size: 10px;
  font-family: inherit;
  color: var(--vp-c-text-3);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 400;
}

.iac-accounts-count {
  font-size: 12px;
  color: var(--vp-c-text-3);
}

.iac-chevron {
  margin-left: auto;
  font-size: 11px;
  color: var(--vp-c-text-3);
}

.iac-card-body {
  border-top: 1px solid var(--vp-c-divider);
  padding: 14px 16px;
  background: var(--vp-c-bg-soft);
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.iac-section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--vp-c-text-3);
  margin-bottom: 8px;
}

.iac-accounts {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.iac-account {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 13px;
}

.iac-account-name {
  display: flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
  min-width: 160px;
}

.iac-badge {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  font-weight: 500;
  font-family: inherit;
}

.iac-badge.write { background: rgba(245, 158, 11, 0.15); color: #B45309; }
.iac-badge.sign  { background: rgba(16, 185, 129, 0.15); color: #047857; }
.iac-badge.pda   { background: rgba(124, 58, 237, 0.12); color: #6D28D9; }

.dark .iac-badge.write { background: rgba(245, 158, 11, 0.2); color: #FCD34D; }
.dark .iac-badge.sign  { background: rgba(16, 185, 129, 0.2); color: #6EE7B7; }
.dark .iac-badge.pda   { background: rgba(124, 58, 237, 0.2); color: #C4B5FD; }

.iac-account-desc {
  color: var(--vp-c-text-3);
  font-size: 12px;
  line-height: 1.4;
}

.iac-events {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.iac-event {
  font-size: 12px;
  font-family: var(--vp-font-family-mono);
  background: rgba(236, 72, 153, 0.1);
  color: #BE185D;
  padding: 2px 8px;
  border-radius: 4px;
}

.dark .iac-event {
  background: rgba(236, 72, 153, 0.15);
  color: #F9A8D4;
}

.iac-notes {
  font-size: 13px;
  color: var(--vp-c-text-2);
  margin: 0;
  line-height: 1.5;
}
</style>
