<template>
  <div class="obv">
    <div class="obv-controls">
      <button
        v-for="(step, i) in steps"
        :key="i"
        :class="['obv-step-btn', { active: activeStep === i }]"
        @click="activeStep = i"
      >
        <code>{{ step.instruction }}</code>
      </button>
    </div>

    <div class="obv-stage">
      <div class="obv-description">
        <div class="obv-instruction-label">
          <span class="obv-tag">{{ steps[activeStep].instruction }}</span>
        </div>
        <p>{{ steps[activeStep].description }}</p>
      </div>

      <div class="obv-book">
        <!-- Asks (sell side) -->
        <div class="obv-side obv-asks">
          <div class="obv-side-label">Asks (Sell)</div>
          <transition-group name="order-row" tag="div" class="obv-orders">
            <div
              v-for="order in steps[activeStep].book.asks"
              :key="order.id"
              :class="['obv-order', 'ask', order.state]"
            >
              <span class="obv-price">{{ order.price }}</span>
              <div class="obv-bar-wrap">
                <div class="obv-bar ask-bar" :style="{ width: order.size * 20 + 'px' }"></div>
              </div>
              <span class="obv-size">{{ order.size }} lots</span>
              <span v-if="order.state" class="obv-state-tag">{{ order.state }}</span>
            </div>
          </transition-group>
        </div>

        <div class="obv-spread">
          <span class="obv-spread-label">spread</span>
          <span class="obv-spread-value">{{ steps[activeStep].spread }}</span>
        </div>

        <!-- Bids (buy side) -->
        <div class="obv-side obv-bids">
          <div class="obv-side-label">Bids (Buy)</div>
          <transition-group name="order-row" tag="div" class="obv-orders">
            <div
              v-for="order in steps[activeStep].book.bids"
              :key="order.id"
              :class="['obv-order', 'bid', order.state]"
            >
              <span class="obv-price">{{ order.price }}</span>
              <div class="obv-bar-wrap">
                <div class="obv-bar bid-bar" :style="{ width: order.size * 20 + 'px' }"></div>
              </div>
              <span class="obv-size">{{ order.size }} lots</span>
              <span v-if="order.state" class="obv-state-tag">{{ order.state }}</span>
            </div>
          </transition-group>
        </div>
      </div>

      <div class="obv-nav">
        <button class="obv-nav-btn" :disabled="activeStep === 0" @click="activeStep--">← Prev</button>
        <span class="obv-step-count">{{ activeStep + 1 }} / {{ steps.length }}</span>
        <button class="obv-nav-btn" :disabled="activeStep === steps.length - 1" @click="activeStep++">Next →</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";

const activeStep = ref(0);

const steps = [
  {
    instruction: "initial state",
    description: "The order book starts empty. No bids, no asks. A market must be registered before orders can be posted.",
    spread: "—",
    book: {
      asks: [],
      bids: [],
    },
  },
  {
    instruction: "post_order (ask)",
    description: "A market maker posts a sell order at price 102 for 5 lots. It rests on the ask side of the book until filled or cancelled.",
    spread: "—",
    book: {
      asks: [{ id: "a1", price: "102", size: 5, state: "new" }],
      bids: [],
    },
  },
  {
    instruction: "post_order (bid)",
    description: "The same maker posts a buy order at price 98 for 5 lots. Now there are resting orders on both sides. The spread is 4 ticks.",
    spread: "4 ticks",
    book: {
      asks: [{ id: "a1", price: "102", size: 5, state: "" }],
      bids: [{ id: "b1", price: "98", size: 5, state: "new" }],
    },
  },
  {
    instruction: "post_order (tighter)",
    description: "A second maker tightens the market, posting a bid at 99 and an ask at 101. The spread narrows to 2 ticks.",
    spread: "2 ticks",
    book: {
      asks: [
        { id: "a2", price: "101", size: 3, state: "new" },
        { id: "a1", price: "102", size: 5, state: "" },
      ],
      bids: [
        { id: "b2", price: "99",  size: 3, state: "new" },
        { id: "b1", price: "98",  size: 5, state: "" },
      ],
    },
  },
  {
    instruction: "market_order",
    description: "A taker sends a market buy order for 3 lots. It matches against the best ask (101) immediately and does not rest on the book. The ask at 101 is fully consumed.",
    spread: "4 ticks",
    book: {
      asks: [
        { id: "a2", price: "101", size: 3, state: "filled" },
        { id: "a1", price: "102", size: 5, state: "" },
      ],
      bids: [
        { id: "b2", price: "99",  size: 3, state: "" },
        { id: "b1", price: "98",  size: 5, state: "" },
      ],
    },
  },
  {
    instruction: "cancel_order",
    description: "The first maker cancels their bid at 98. The order is removed from the book and their reserved quote tokens return to their available balance.",
    spread: "3 ticks",
    book: {
      asks: [
        { id: "a1", price: "102", size: 5, state: "" },
      ],
      bids: [
        { id: "b2", price: "99",  size: 3, state: "" },
        { id: "b1", price: "98",  size: 5, state: "cancelled" },
      ],
    },
  },
  {
    instruction: "batch_replace",
    description: "The second maker uses batch_replace to cancel their remaining orders and post fresh quotes atomically — new ask at 101 for 4 lots, new bid at 99 for 4 lots. No gap between cancel and post.",
    spread: "2 ticks",
    book: {
      asks: [
        { id: "a3", price: "101", size: 4, state: "new" },
        { id: "a1", price: "102", size: 5, state: "" },
      ],
      bids: [
        { id: "b3", price: "99",  size: 4, state: "new" },
      ],
    },
  },
];
</script>

<style scoped>
.obv {
  margin: 1.5rem 0;
  font-family: inherit;
}

.obv-controls {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 1rem;
}

.obv-step-btn {
  padding: 5px 12px;
  border-radius: 6px;
  border: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg);
  color: var(--vp-c-text-2);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.obv-step-btn.active {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
  background: var(--vp-c-bg-soft);
}

.obv-stage {
  border: 1px solid var(--vp-c-divider);
  border-radius: 10px;
  overflow: hidden;
}

.obv-description {
  padding: 14px 18px;
  background: var(--vp-c-bg-soft);
  border-bottom: 1px solid var(--vp-c-divider);
}

.obv-instruction-label {
  margin-bottom: 6px;
}

.obv-tag {
  font-family: var(--vp-font-family-mono);
  font-size: 12px;
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  padding: 2px 8px;
  border-radius: 4px;
}

.obv-description p {
  margin: 0;
  font-size: 14px;
  color: var(--vp-c-text-2);
  line-height: 1.5;
}

.obv-book {
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 0;
}

.obv-side-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--vp-c-text-3);
  margin-bottom: 6px;
}

.obv-orders {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 32px;
}

.obv-asks .obv-orders {
  flex-direction: column-reverse;
}

.obv-order {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 5px 10px;
  border-radius: 5px;
  font-size: 13px;
  transition: all 0.3s;
  border: 1px solid transparent;
}

.obv-order.ask {
  background: rgba(239, 68, 68, 0.06);
}

.obv-order.bid {
  background: rgba(34, 197, 94, 0.06);
}

.obv-order.new {
  border-color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
}

.obv-order.filled {
  opacity: 0.4;
  text-decoration: line-through;
  border-color: #0EA5E9;
  background: rgba(14, 165, 233, 0.08);
}

.obv-order.cancelled {
  opacity: 0.35;
  text-decoration: line-through;
}

.obv-price {
  font-family: var(--vp-font-family-mono);
  font-size: 13px;
  font-weight: 600;
  min-width: 36px;
  color: var(--vp-c-text-1);
}

.obv-bar-wrap {
  flex: 1;
  max-width: 120px;
}

.obv-bar {
  height: 6px;
  border-radius: 3px;
  transition: width 0.3s ease;
}

.ask-bar { background: rgba(239, 68, 68, 0.5); }
.bid-bar { background: rgba(34, 197, 94, 0.5); }

.obv-size {
  font-size: 12px;
  color: var(--vp-c-text-3);
  min-width: 50px;
}

.obv-state-tag {
  font-size: 11px;
  padding: 1px 7px;
  border-radius: 10px;
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  font-weight: 500;
}

.obv-order.filled .obv-state-tag {
  background: rgba(14, 165, 233, 0.15);
  color: #0EA5E9;
}

.obv-order.cancelled .obv-state-tag {
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-3);
}

.obv-spread {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  margin: 6px 0;
  border-top: 1px dashed var(--vp-c-divider);
  border-bottom: 1px dashed var(--vp-c-divider);
}

.obv-spread-label {
  font-size: 11px;
  color: var(--vp-c-text-3);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.obv-spread-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--vp-c-text-1);
  font-family: var(--vp-font-family-mono);
}

.obv-nav {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 12px;
  border-top: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg-soft);
}

.obv-nav-btn {
  padding: 5px 14px;
  border-radius: 6px;
  border: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.obv-nav-btn:hover:not(:disabled) {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.obv-nav-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.obv-step-count {
  font-size: 13px;
  color: var(--vp-c-text-3);
}

.order-row-enter-active,
.order-row-leave-active {
  transition: all 0.3s ease;
}
.order-row-enter-from {
  opacity: 0;
  transform: translateX(-10px);
}
.order-row-leave-to {
  opacity: 0;
  transform: translateX(10px);
}
</style>
