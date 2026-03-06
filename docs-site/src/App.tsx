import Architecture from './components/architecture/Architecture';
import Explorer from './components/explorer/Explorer';
import Hero from './components/Hero';
import Navbar from './components/Navbar';
import QuickStart from './components/QuickStart';
import ProtocolReference from './components/reference/ProtocolReference';
import Section from './components/Section';
import VariantLookup from './components/VariantLookup';

export default function App() {
  return (
    <div id="top" className="relative min-h-screen overflow-x-hidden">
      <a href="#content" className="skip-link">
        Skip to content
      </a>
      <div className="noise-overlay" />
      <Navbar />
      <Hero />

      <main id="content">
        <Section
          id="quick-start"
          title="Quick Start"
          description="This is the shortest path from raw Solana payloads to a lifecycle decision you can trust."
          index={0}
        >
          <QuickStart />
        </Section>

        <Section
          id="playground"
          title="Lifecycle Playground"
          description="Use the canonical event buttons to explore the state machine, terminal-state rules, and snapshot delta behavior."
          index={1}
        >
          <Explorer />
        </Section>

        <Section
          id="variant-lookup"
          title="Variant Lookup"
          description="Check raw instruction and event names against the crate’s canonical mapping without pretending to validate the full payload."
          index={2}
        >
          <VariantLookup />
        </Section>

        <Section
          id="protocols"
          title="Protocol Differences"
          description="Keep the protocol-specific quirks in one place: raw variant names, program IDs, close handling, and Kamino correlation requirements."
          index={3}
        >
          <ProtocolReference />
        </Section>

        <Section
          id="reliability"
          title="Why It’s Reliable"
          description="The crate stays simple on purpose: pure logic in, lifecycle decisions out, with protocol quirks and test coverage made explicit."
          index={4}
        >
          <Architecture />
        </Section>
      </main>

      <footer className="relative border-t border-border px-4 py-12 text-center sm:px-8">
        <div className="absolute inset-0 pointer-events-none bg-gradient-to-t from-accent/[0.02] to-transparent" />
        <p className="relative text-sm text-dim">
          <code className="font-mono text-text">defi-tracker-lifecycle</code> is a pure-logic
          crate for DeFi order lifecycle tracking on Solana.
        </p>
        <p className="relative mt-2 text-sm text-dim/70">
          <a
            href="https://github.com/ohaddahan/defi-tracker-lifecycle"
            className="focus-ring ui-transition text-accent/70 hover:text-accent"
          >
            Source on GitHub
          </a>
          <span className="mx-2">·</span>
          Built with React, Tailwind, and the crate’s WASM bindings
        </p>
      </footer>
    </div>
  );
}
