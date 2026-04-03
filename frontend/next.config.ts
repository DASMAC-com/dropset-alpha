import path from "node:path";
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactCompiler: true,
  transpilePackages: ["@/ts-sdk"],
  turbopack: {
    root: path.join(import.meta.dirname, ".."),
    resolveAlias: {
      "@/ts-sdk": "../ts-sdk/src/index.ts",
      "@/ts-sdk/*": "../ts-sdk/src/*",
    },
  },
  logging:
    process.env.NODE_ENV === "development" ||
    process.env.NODE_ENV === "test" ||
    process.env.VERCEL_ENV === "preview" ||
    process.env.VERCEL_ENV === "development"
      ? {
          fetches: {
            fullUrl: true,
          },
        }
      : undefined,
  typescript: {
    tsconfigPath: "tsconfig.json",
  },
};

export default nextConfig;
