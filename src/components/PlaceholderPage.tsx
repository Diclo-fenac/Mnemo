import mnemoMark from "../assets/mnemo-mark.svg";

type PlaceholderPageProps = { eyebrow: string; title: string; copy: string; status: string };

export function PlaceholderPage({ eyebrow, title, copy, status }: PlaceholderPageProps) {
  return <section className="page"><p className="eyebrow">{eyebrow}</p><h1 className="page-title">{title}</h1><p className="page-copy">{copy}</p><div className="stage-card"><strong>{status}</strong><p>This surface is wired into the final application shell and will become functional as its Milestone 2-5 services land.</p></div><img className="empty-symbol" src={mnemoMark} alt="" /></section>;
}
