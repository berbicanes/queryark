export interface ChangelogEntry {
  version: string;
  date: string;
  highlights: string[];
  category: 'feature' | 'fix' | 'improvement';
}

export const CHANGELOG: ChangelogEntry[] = [
  {
    version: '0.2.1',
    date: '2026-02-23',
    highlights: [
      'Add product website with automated deployment',
      'Fix release workflow permissions',
    ],
    category: 'fix',
  },
  {
    version: '0.2.0',
    date: '2026-02-23',
    highlights: [
      'Crash reporting with Sentry (opt-in)',
      'Anonymous usage telemetry (opt-out by default)',
      'In-app "What\'s New" changelog after updates',
      'Automatic config backup with restore support',
      'End-to-end test scaffolding with Playwright',
      'Auto-suggest indexes for slow queries',
      'Query result charts (bar, line, pie)',
      'Side-by-side result comparison',
      'Parameterized query execution',
      'Query profiling dashboard with optimization hints',
    ],
    category: 'feature',
  },
  {
    version: '0.1.0',
    date: '2026-01-01',
    highlights: [
      'Initial release with 17 database engine support',
      'Query editor with CodeMirror 6',
      'Inline cell editing, row insertion/deletion',
      'Connection management with SSH tunneling and SSL',
      'ER diagram viewer and visual query builder',
      'Table/data diff and migration generator',
    ],
    category: 'feature',
  },
];
