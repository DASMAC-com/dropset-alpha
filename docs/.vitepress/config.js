export default {
  title: "Dropset",
  description:
    "Courtesy of Distributed Atomic State Machine Algorithms Corporation (DASMAC)",
  head: [
    [
      "link",
      {
        rel: "icon",
        href: "/favicon-light.png",
        media: "(prefers-color-scheme: light)",
      },
    ],
    [
      "link",
      {
        rel: "icon",
        href: "/favicon-dark.png",
        media: "(prefers-color-scheme: dark)",
      },
    ],
    ["link", { rel: "apple-touch-icon", href: "/favicon-light.png" }],
    ["meta", { property: "og:site_name", content: "DASMAC" }],
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:url", content: "https://docs.dropset.io/" }],
    ["meta", { property: "og:title", content: "Dropset Docs" }],
    [
      "meta",
      {
        property: "og:description",
        content:
          "Courtesy of Distributed Atomic State Machine Algorithms Corporation (DASMAC)",
      },
    ],
    [
      "meta",
      {
        property: "og:image",
        content: "https://docs.dropset.io/dasmac-banner.png",
      },
    ],
    ["meta", { name: "twitter:card", content: "summary_large_image" }],
    ["meta", { name: "twitter:title", content: "Dropset Docs" }],
    [
      "meta",
      {
        name: "twitter:description",
        content:
          "Courtesy of Distributed Atomic State Machine Algorithms Corporation (DASMAC)",
      },
    ],
    [
      "meta",
      {
        name: "twitter:image",
        content: "https://docs.dropset.io/dasmac-banner.png",
      },
    ],
  ],
  srcDir: "src",
  themeConfig: {
    outline: "deep",
    editLink: {
      pattern:
        "https://github.com/DASMAC-com/dropset-alpha/blob/main/docs/src/:path",
      text: "Contribute to this page",
    },
    sidebar: [
      { text: "Welcome", link: "/" },
      {
        collapsed: false,
        text: "Introduction",
        items: [
          { text: "What is Dropset?", link: "/introduction/what-is-dropset" },
          { text: "Core Concepts", link: "/introduction/core-concepts" },
        ],
      },
      {
        collapsed: false,
        text: "Architecture",
        items: [
          { text: "Overview", link: "/architecture/overview" },
          { text: "Program Structure", link: "/architecture/program-structure" },
          { text: "On-Chain Accounts", link: "/architecture/accounts" },
        ],
      },
      {
        collapsed: false,
        text: "Quickstart",
        items: [
          { text: "Getting Started", link: "/quickstart/getting-started" },
        ],
      },
      {
        collapsed: false,
        text: "TypeScript SDK",
        items: [
          { text: "Overview", link: "/sdk/overview" },
          { text: "Connect to a Market", link: "/sdk/connect-to-market" },
          { text: "Post an Order", link: "/sdk/post-order" },
          { text: "Price Utilities", link: "/sdk/price-utils" },
        ],
      },
      {
        collapsed: false,
        text: "Services",
        items: [
          { text: "Faucet, Maker & Taker", link: "/services/overview" },
        ],
      },
    ],
  },
};
