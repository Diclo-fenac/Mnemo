import { type FormEvent, useState } from "react";
import { Clock3, Network, Search, Settings, Sparkles } from "lucide-react";
import { NavLink, useNavigate } from "react-router-dom";
import { BrandMark } from "./BrandMark";
import { CaptureControl } from "./CaptureControl";

const links = [
  { to: "/", label: "Memory", icon: Sparkles },
  { to: "/timeline", label: "Timeline", icon: Clock3 },
  { to: "/graph", label: "Memory graph", icon: Network },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  const [query, setQuery] = useState("");
  const navigate = useNavigate();

  function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalized = query.trim();
    if (!normalized) return;
    navigate(`/search?q=${encodeURIComponent(normalized)}`);
  }

  return <aside className="sidebar">
    <div className="brand" aria-label="Mnemo"><BrandMark variant="wordmark" size={34} /></div>
    <form
      className="sidebar-search"
      onSubmit={submitSearch}
      onClick={() => {
        if (window.matchMedia("(max-width: 900px)").matches) navigate("/search");
      }}
    >
      <Search size={16} aria-hidden="true" />
      <input
        data-sidebar-search
        aria-label="Search your memory"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Search memory"
      />
      <kbd>Ctrl K</kbd>
    </form>
    <nav className="nav">{links.map(({ to, label, icon: Icon }) => <NavLink key={to} to={to} end={to === "/"} className="nav-link"><Icon size={18} /><span>{label}</span></NavLink>)}</nav>
    <CaptureControl />
  </aside>;
}
