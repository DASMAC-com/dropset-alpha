import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactCompiler: true,
  typedRoutes: true,
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
