import Navbar from './components/Navbar';
import Hero from './components/Hero';
import Section from './components/Section';
import Explorer from './components/explorer/Explorer';
import JsonClassifier from './components/JsonClassifier';
import ProtocolReference from './components/reference/ProtocolReference';
import PipelineVisualizer from './components/pipeline/PipelineVisualizer';
import Architecture from './components/architecture/Architecture';

export default function App() {
  return (
    <div className="min-h-screen relative">
      <div className="noise-overlay" />
      <Navbar />
      <Hero />

      <Section id="explorer" title="State Machine Explorer" index={0}>
        <Explorer />
      </Section>

      <Section id="classifier" title="JSON Classifier" index={1}>
        <JsonClassifier />
      </Section>

      <Section id="reference" title="Protocol Reference" index={2}>
        <ProtocolReference />
      </Section>

      <Section id="pipeline" title="Pipeline Visualizer" index={3}>
        <PipelineVisualizer />
      </Section>

      <Section id="architecture" title="Architecture" index={4}>
        <Architecture />
      </Section>

      <footer className="relative py-12 px-4 text-center border-t border-border">
        <div className="absolute inset-0 bg-gradient-to-t from-accent/[0.02] to-transparent pointer-events-none" />
        <p className="text-xs text-dim relative">
          defi-tracker-lifecycle — Pure-logic crate for DeFi order lifecycle tracking on Solana.
        </p>
        <p className="text-xs text-dim/60 mt-2 relative">
          <a
            href="https://github.com/ohaddahan/defi-tracker-lifecycle"
            className="text-accent/60 hover:text-accent transition-colors"
          >
            Source on GitHub
          </a>
          <span className="mx-2">·</span>
          Built with React + Tailwind + framer-motion
        </p>
      </footer>
    </div>
  );
}
