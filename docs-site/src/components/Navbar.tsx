const NAV_ITEMS = [
  { label: 'Explorer', href: '#explorer' },
  { label: 'Classifier', href: '#classifier' },
  { label: 'Reference', href: '#reference' },
  { label: 'Pipeline', href: '#pipeline' },
  { label: 'Architecture', href: '#architecture' },
];

export default function Navbar() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-bg/80 backdrop-blur-md border-b border-border">
      <div className="mx-auto max-w-6xl flex items-center justify-between px-4 sm:px-8 h-14">
        <a href="#" className="font-semibold text-sm text-text hover:text-accent transition-colors">
          defi-tracker-lifecycle
        </a>
        <div className="hidden sm:flex items-center gap-6">
          {NAV_ITEMS.map((item) => (
            <a
              key={item.href}
              href={item.href}
              className="text-xs text-dim hover:text-text transition-colors"
            >
              {item.label}
            </a>
          ))}
          <a
            href="https://github.com/ohaddahan/defi-tracker-lifecycle"
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs text-dim hover:text-text transition-colors"
          >
            GitHub
          </a>
        </div>
      </div>
    </nav>
  );
}
