"use client";

import Image from "next/image";
import Link from "next/link";
import { usePathname } from "next/navigation";

const NAV_ITEMS = [
  { label: "Info", href: "/info" },
  { label: "Repo", href: "/repo" },
  { label: "Team", href: "/team" },
];

export function Header() {
  const pathname = usePathname();

  return (
    <header className="sticky top-0 z-50 border-border border-b bg-background">
      <div className="mx-auto flex h-14 max-w-6xl items-center gap-2 px-6">
        <Link
          href="/"
          className="flex shrink-0 items-center gap-2 font-semibold text-base text-foreground no-underline"
        >
          <Image
            src="/dropset.svg"
            alt="Dropset"
            width={36}
            height={36}
            className="h-12 w-auto rounded-md"
          />
        </Link>

        <nav className="flex items-center gap-2">
          {NAV_ITEMS.map(({ label, href }) => (
            <Link
              key={href}
              href={href}
              className={`rounded-md px-3 py-1.5 text-sm no-underline transition-colors ${
                pathname === href
                  ? "bg-muted text-foreground"
                  : "text-muted-fg hover:bg-muted hover:text-foreground"
              }`}
            >
              {label}
            </Link>
          ))}
        </nav>
      </div>
    </header>
  );
}
