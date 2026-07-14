import { Clock3, Network, Search, Settings } from "lucide-react";
import { NavLink } from "react-router-dom";
import mnemoMark from "../assets/mnemo-mark.svg";

const links = [
  { to: "/", label: "Timeline", icon: Clock3 },
  { to: "/search", label: "Search", icon: Search },
  { to: "/graph", label: "Memory graph", icon: Network },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  return <aside className="sidebar">
    <div className="brand"><div className="brand-mark"><img src={mnemoMark} alt="" /></div><span>mnemo</span></div>
    <nav className="nav">{links.map(({ to, label, icon: Icon }) => <NavLink key={to} to={to} end={to === "/"} className="nav-link"><Icon size={18} /><span>{label}</span></NavLink>)}</nav>
  </aside>;
}
