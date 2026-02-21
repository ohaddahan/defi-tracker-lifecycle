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
    <div className="min-h-screen">
      <Navbar />
      <Hero />

      <Section id="explorer" title="State Machine Explorer">
        <Explorer />
      </Section>

      <Section id="classifier" title="JSON Classifier">
        <JsonClassifier />
      </Section>

      <Section id="reference" title="Protocol Reference">
        <ProtocolReference />
      </Section>

      <Section id="pipeline" title="Pipeline Visualizer">
        <PipelineVisualizer />
      </Section>

      <Section id="architecture" title="Architecture">
        <Architecture />
      </Section>

      <footer className="py-8 px-4 text-center text-xs text-dim border-t border-border">
        <p>
          defi-tracker-lifecycle — Pure-logic crate for DeFi order lifecycle tracking.{' '}
          <a
            href="https://github.com/ohaddahan/defi-tracker-lifecycle"
            className="text-accent hover:underline"
          >
            Source on GitHub
          </a>
        </p>
      </footer>
    </div>
  );
}
