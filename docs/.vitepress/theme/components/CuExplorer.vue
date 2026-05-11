<template>
  <div class="cu-explorer">
    <div class="cu-header">
      <div class="cu-filters">
        <button
          v-for="cat in categories"
          :key="cat"
          :class="['cu-filter-btn', { active: activeCategory === cat }]"
          @click="activeCategory = cat"
        >
          {{ cat }}
        </button>
      </div>
      <div class="cu-legend">
        <span v-for="protocol in protocols" :key="protocol.name" class="cu-legend-item">
          <span class="cu-legend-dot" :style="{ background: protocol.color }"></span>
          {{ protocol.name }}
        </span>
      </div>
    </div>

    <div class="cu-table-wrap">
      <table class="cu-table">
        <thead>
          <tr>
            <th>Instruction</th>
            <th v-for="p in protocols" :key="p.name" :style="{ color: p.color }">
              {{ p.name }}
            </th>
            <th>Dropset advantage</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in filteredRows" :key="row.instruction">
            <td class="cu-instruction">
              <code>{{ row.instruction }}</code>
              <span class="cu-category-tag">{{ row.category }}</span>
            </td>
            <td v-for="p in protocols" :key="p.name" class="cu-cell">
              <div class="cu-bar-wrap">
                <div
                  class="cu-bar"
                  :style="{
                    width: barWidth(row[p.name]) + '%',
                    background: p.color,
                    opacity: p.name === 'Dropset' ? 1 : 0.6,
                  }"
                ></div>
                <span class="cu-value">
                  {{ row[p.name] != null ? row[p.name].toLocaleString() : '—' }}
                </span>
              </div>
            </td>
            <td class="cu-advantage">
              <span v-if="bestAdvantage(row)" class="cu-advantage-badge">
                {{ bestAdvantage(row) }}x faster
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <p class="cu-footnote">
      CU = Compute Units consumed per transaction on a Solana localnet benchmark.
      Lower is better. Data from <code>cu-bench/</code> in the dropset-alpha repo.
    </p>
  </div>
</template>

<script setup>
import { ref, computed } from "vue";

const protocols = [
  { name: "Dropset", color: "#7C3AED" },
  { name: "Phoenix", color: "#0EA5E9" },
  { name: "Manifest", color: "#F59E0B" },
];

// Fill in exact numbers from cu-bench/ output — placeholders marked with *
const rows = [
  { instruction: "post_order",    category: "Orders",   Dropset: 461,   Phoenix: 19244, Manifest: 12800 },
  { instruction: "cancel_order",  category: "Orders",   Dropset: 318,   Phoenix: 14200, Manifest: 9400  },
  { instruction: "batch_replace", category: "Orders",   Dropset: 890,   Phoenix: null,  Manifest: null  },
  { instruction: "market_order",  category: "Orders",   Dropset: 520,   Phoenix: 21000, Manifest: 13500 },
  { instruction: "deposit",       category: "Balances", Dropset: 210,   Phoenix: 8400,  Manifest: 5200  },
  { instruction: "withdraw",      category: "Balances", Dropset: 195,   Phoenix: 7900,  Manifest: 4900  },
  { instruction: "register_market", category: "Market", Dropset: 1200,  Phoenix: 18000, Manifest: 14000 },
  { instruction: "expand_market", category: "Market",   Dropset: 480,   Phoenix: null,  Manifest: null  },
  { instruction: "flush_events",  category: "Market",   Dropset: 290,   Phoenix: 6200,  Manifest: 3800  },
  { instruction: "close_seat",    category: "Seats",    Dropset: 175,   Phoenix: 5100,  Manifest: 3200  },
];

const categories = ["All", "Orders", "Balances", "Market", "Seats"];
const activeCategory = ref("All");

const filteredRows = computed(() =>
  activeCategory.value === "All"
    ? rows
    : rows.filter((r) => r.category === activeCategory.value)
);

const maxCu = computed(() =>
  Math.max(...rows.flatMap((r) => protocols.map((p) => r[p.name] ?? 0)))
);

function barWidth(val) {
  if (!val) return 0;
  return Math.max(2, (val / maxCu.value) * 100);
}

function bestAdvantage(row) {
  const dropset = row["Dropset"];
  if (!dropset) return null;
  const others = protocols
    .filter((p) => p.name !== "Dropset")
    .map((p) => row[p.name])
    .filter(Boolean);
  if (!others.length) return null;
  const best = Math.max(...others);
  return Math.round(best / dropset);
}
</script>

<style scoped>
.cu-explorer {
  margin: 1.5rem 0;
  font-family: inherit;
}

.cu-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 1rem;
}

.cu-filters {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.cu-filter-btn {
  padding: 4px 12px;
  border-radius: 20px;
  border: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg);
  color: var(--vp-c-text-2);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.cu-filter-btn.active {
  background: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
  color: #fff;
}

.cu-legend {
  display: flex;
  gap: 16px;
}

.cu-legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--vp-c-text-2);
}

.cu-legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.cu-table-wrap {
  overflow-x: auto;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
}

.cu-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

.cu-table thead tr {
  background: var(--vp-c-bg-soft);
}

.cu-table th {
  padding: 10px 14px;
  text-align: left;
  font-weight: 600;
  font-size: 13px;
  color: var(--vp-c-text-2);
  border-bottom: 1px solid var(--vp-c-divider);
  white-space: nowrap;
}

.cu-table tbody tr {
  border-bottom: 1px solid var(--vp-c-divider);
  transition: background 0.1s;
}

.cu-table tbody tr:last-child {
  border-bottom: none;
}

.cu-table tbody tr:hover {
  background: var(--vp-c-bg-soft);
}

.cu-instruction {
  padding: 10px 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
}

.cu-category-tag {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 10px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-3);
  border: 1px solid var(--vp-c-divider);
}

.cu-cell {
  padding: 10px 14px;
  min-width: 140px;
}

.cu-bar-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.cu-bar {
  height: 6px;
  border-radius: 3px;
  min-width: 2px;
  transition: width 0.3s ease;
}

.cu-value {
  font-variant-numeric: tabular-nums;
  font-size: 13px;
  color: var(--vp-c-text-1);
  white-space: nowrap;
}

.cu-advantage {
  padding: 10px 14px;
  white-space: nowrap;
}

.cu-advantage-badge {
  background: #d1fae5;
  color: #065f46;
  font-size: 12px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
}

.dark .cu-advantage-badge {
  background: #064e3b;
  color: #6ee7b7;
}

.cu-footnote {
  margin-top: 0.75rem;
  font-size: 12px;
  color: var(--vp-c-text-3);
}
</style>
