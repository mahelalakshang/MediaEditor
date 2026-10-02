import { LayoutGrid, List, Search } from 'lucide-react';

export type KindFilter = 'ALL' | 'IMAGE' | 'VIDEO';
export type StatusFilter = 'ALL' | 'PROCESSED' | 'PROCESSING' | 'FAILED';
export type SortKey = 'newest' | 'oldest' | 'name';
export type ViewMode = 'grid' | 'list';

export interface LibraryFilters {
  query: string;
  kind: KindFilter;
  status: StatusFilter;
  sort: SortKey;
  view: ViewMode;
}

const KINDS: { value: KindFilter; label: string }[] = [
  { value: 'ALL', label: 'All' },
  { value: 'IMAGE', label: 'Images' },
  { value: 'VIDEO', label: 'Videos' },
];

export function Toolbar({
  filters,
  onChange,
  total,
  shown,
}: {
  filters: LibraryFilters;
  onChange: (patch: Partial<LibraryFilters>) => void;
  total: number;
  shown: number;
}) {
  return (
    <div className="toolbar">
      <div className="search">
        <Search size={16} />
        <input
          type="search"
          placeholder="Search files"
          value={filters.query}
          onChange={(e) => onChange({ query: e.target.value })}
          aria-label="Search files"
        />
      </div>

      <div className="segmented" role="group" aria-label="Filter by type">
        {KINDS.map((k) => (
          <button
            key={k.value}
            className={filters.kind === k.value ? 'active' : ''}
            aria-pressed={filters.kind === k.value}
            onClick={() => onChange({ kind: k.value })}
          >
            {k.label}
          </button>
        ))}
      </div>

      <select
        className="select"
        value={filters.status}
        onChange={(e) => onChange({ status: e.target.value as StatusFilter })}
        aria-label="Filter by status"
      >
        <option value="ALL">Any status</option>
        <option value="PROCESSED">Ready</option>
        <option value="PROCESSING">Processing</option>
        <option value="FAILED">Failed</option>
      </select>

      <select
        className="select"
        value={filters.sort}
        onChange={(e) => onChange({ sort: e.target.value as SortKey })}
        aria-label="Sort"
      >
        <option value="newest">Newest first</option>
        <option value="oldest">Oldest first</option>
        <option value="name">Name</option>
      </select>

      <span className="toolbar-count">
        {shown === total ? `${total} items` : `${shown} of ${total}`}
      </span>

      <div className="segmented view-toggle" role="group" aria-label="View mode">
        <button
          className={filters.view === 'grid' ? 'active' : ''}
          aria-pressed={filters.view === 'grid'}
          aria-label="Grid view"
          onClick={() => onChange({ view: 'grid' })}
        >
          <LayoutGrid size={16} />
        </button>
        <button
          className={filters.view === 'list' ? 'active' : ''}
          aria-pressed={filters.view === 'list'}
          aria-label="List view"
          onClick={() => onChange({ view: 'list' })}
        >
          <List size={16} />
        </button>
      </div>
    </div>
  );
}
