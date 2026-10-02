import { useMemo, useState } from 'react';
import { FolderOpen, SearchX, Upload } from 'lucide-react';
import type { MediaAsset } from '../types';
import { Gallery, GallerySkeleton } from './Gallery';
import { Toolbar } from './Toolbar';
import type { LibraryFilters } from './Toolbar';

const DEFAULT_FILTERS: LibraryFilters = {
  query: '',
  kind: 'ALL',
  status: 'ALL',
  sort: 'newest',
  view: 'grid',
};

function applyFilters(assets: MediaAsset[], f: LibraryFilters): MediaAsset[] {
  const q = f.query.trim().toLowerCase();
  const out = assets.filter((a) => {
    if (q && !a.originalFilename.toLowerCase().includes(q)) return false;
    if (f.kind !== 'ALL' && a.kind !== f.kind) return false;
    if (f.status === 'PROCESSING') return a.status === 'PROCESSING' || a.status === 'UPLOADED';
    if (f.status !== 'ALL' && a.status !== f.status) return false;
    return true;
  });
  const byDate = (a: MediaAsset, b: MediaAsset) => a.createdAt.localeCompare(b.createdAt);
  if (f.sort === 'newest') out.sort((a, b) => byDate(b, a));
  else if (f.sort === 'oldest') out.sort(byDate);
  else out.sort((a, b) => a.originalFilename.localeCompare(b.originalFilename));
  return out;
}

export function Library({
  assets,
  loading,
  onSelect,
  onUpload,
}: {
  assets: MediaAsset[];
  loading: boolean;
  onSelect: (id: string) => void;
  onUpload: () => void;
}) {
  const [filters, setFilters] = useState<LibraryFilters>(DEFAULT_FILTERS);
  const visible = useMemo(() => applyFilters(assets, filters), [assets, filters]);

  return (
    <section>
      <div className="page-head">
        <div>
          <h1>Library</h1>
          <p className="muted">Everything you've uploaded and processed.</p>
        </div>
      </div>

      {loading ? (
        <GallerySkeleton />
      ) : assets.length === 0 ? (
        <div className="empty">
          <span className="empty-icon">
            <FolderOpen size={28} />
          </span>
          <h2>No media yet</h2>
          <p className="muted">Upload an image or video and we'll generate its thumbnail.</p>
          <button className="btn btn-primary" onClick={onUpload}>
            <Upload size={16} />
            Upload media
          </button>
        </div>
      ) : (
        <>
          <Toolbar
            filters={filters}
            onChange={(patch) => setFilters((prev) => ({ ...prev, ...patch }))}
            total={assets.length}
            shown={visible.length}
          />
          {visible.length === 0 ? (
            <div className="empty">
              <span className="empty-icon">
                <SearchX size={28} />
              </span>
              <h2>No matches</h2>
              <p className="muted">Try a different search or clear the filters.</p>
              <button className="btn" onClick={() => setFilters(DEFAULT_FILTERS)}>
                Clear filters
              </button>
            </div>
          ) : (
            <Gallery assets={visible} view={filters.view} onSelect={onSelect} />
          )}
        </>
      )}
    </section>
  );
}
