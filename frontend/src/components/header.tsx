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
    <header className="header">
      <div className="header-inner">
        <Link href="/" className="header-logo">
          <Image
            src="/dropset.svg"
            alt="Dropset"
            width={36}
            height={36}
            className="header-logo-icon"
          />
          <span className="header-logo-text">Dropset</span>
        </Link>

        <nav className="header-nav">
          {NAV_ITEMS.map(({ label, href }) => (
            <Link
              key={href}
              href={href}
              className={`header-nav-link ${pathname === href ? "active" : ""}`}
            >
              {label}
            </Link>
          ))}
        </nav>
      </div>
    </header>
  );
}
