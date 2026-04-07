import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { fetchAllMarkets } from "@/lib/queries/fetch-all-markets";
import { resolveSlug } from "@/lib/slug";
import { MarketView } from "./market-view";

type Props = {
  params: Promise<{ slug: string }>;
};

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  const markets = await fetchAllMarkets();
  const address =
    resolveSlug(
      slug,
      markets.map((m) => m.address),
    ) ?? notFound();

  return {
    title: `dropset – market ${address}`,
    description: `A dropset alpha market at address ${address}`,
  };
}

export default async function MarketPage({ params }: Props) {
  const { slug } = await params;
  const markets = await fetchAllMarkets();
  const address = resolveSlug(
    slug,
    markets.map((m) => m.address),
  );

  if (!address) notFound();

  return <MarketView address={address} />;
}
