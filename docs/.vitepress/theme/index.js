import DefaultTheme from "vitepress/theme";
import CuExplorer from "./components/CuExplorer.vue";
import OrderBookViz from "./components/OrderBookViz.vue";
import InstructionCards from "./components/InstructionCards.vue";

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component("CuExplorer", CuExplorer);
    app.component("OrderBookViz", OrderBookViz);
    app.component("InstructionCards", InstructionCards);
  },
};
