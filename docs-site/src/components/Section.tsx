import type { ReactNode } from 'react';

interface Props {
  id: string;
  title: string;
  children: ReactNode;
  className?: string;
}

export default function Section({ id, title, children, className = '' }: Props) {
  return (
    <section id={id} className={`py-16 px-4 sm:px-8 ${className}`}>
      <div className="mx-auto max-w-6xl">
        <h2 className="text-2xl font-bold mb-8 text-text">{title}</h2>
        {children}
      </div>
    </section>
  );
}
