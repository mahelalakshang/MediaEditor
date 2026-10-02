import { Download, Film, Image as ImageIcon } from 'lucide-react';
import { mediaUrl } from '../api';
import { baseName, relativeTime } from '../lib/format';
import type { MediaAsset } from '../types';
import { StatusBadge } from './StatusBadge';
import type { ViewMode } from './Toolbar';

export function Gallery({
  assets,
  view,
  onSelect,
}: {
  assets: MediaAsset[];
  view: ViewMode;
  onSelect: (id: string) => void;
}) {
  return (
    <div className={view === 'grid' ? 'gallery-grid' : 'gallery-list'}>
      {assets.map((asset) => {
        const thumb = asset.renditions.find(
          (r) => r.renditionType === 'THUMBNAIL' && r.isSelected,
        );
        const KindIcon = asset.kind === 'VIDEO' ? Film : ImageIcon;
        return (
          <article key={asset.id} className="asset-card">
            <button
              className="asset-card-main"
              onClick={() => onSelect(asset.id)}
              aria-label={`Open ${asset.originalFilename}`}
            >
              <div className="asset-card-thumb">
                {thumb ? (
                  <img src={mediaUrl(thumb.path)} alt="" loading="lazy" />
                ) : (
                  <div className="thumb-placeholder">
                    <KindIcon size={28} />
                  </div>
                )}
                <span className="kind-tag">
                  <KindIcon size={12} />
                  {asset.kind === 'VIDEO' ? 'Video' : 'Image'}
                </span>
              </div>
              <div className="asset-card-info">
                <span className="asset-card-name" title={asset.originalFilename}>
                  {asset.originalFilename}
                </span>
                <div className="asset-card-meta">
                  <StatusBadge status={asset.status} />
                  <span className="muted">{relativeTime(asset.createdAt)}</span>
                </div>
              </div>
            </button>
            {thumb && (
              <a
                className="card-action"
                href={mediaUrl(thumb.path)}
                download={`${baseName(asset.originalFilename)}_thumbnail.jpg`}
                aria-label="Download thumbnail"
                title="Download thumbnail"
              >
                <Download size={15} />
              </a>
            )}
          </article>
        );
      })}
    </div>
  );
}

export function GallerySkeleton() {
  return (
    <div className="gallery-grid" aria-hidden>
      {Array.from({ length: 8 }).map((_, i) => (
        <div key={i} className="asset-card skeleton-card">
          <div className="asset-card-thumb skeleton" />
          <div className="asset-card-info">
            <div className="skeleton skeleton-line" />
            <div className="skeleton skeleton-line short" />
          </div>
        </div>
      ))}
    </div>
  );
}
