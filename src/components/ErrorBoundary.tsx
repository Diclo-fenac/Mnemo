import { Component, type ErrorInfo, type ReactNode } from "react";

type Props = { children: ReactNode };
type State = { error: Error | null };

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State { return { error }; }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Unhandled UI error", error, info);
  }

  render() {
    if (this.state.error) {
      return <section className="page"><p className="eyebrow">Mnemo</p><h1 className="page-title">This view could not load.</h1><p className="page-copy">{this.state.error.message}</p><button className="primary-button" onClick={() => this.setState({ error: null })}>Try again</button></section>;
    }
    return this.props.children;
  }
}
