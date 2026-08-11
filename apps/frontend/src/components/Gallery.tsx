import { mediaUrl } from '../api';
import type { MediaAsset } from '../types';
import { StatusBadge } from './StatusBadge';

export function Gallery({
  assets,
  onSelect,
}: {
  assets: MediaAsset[];
  onSelect: (id: string) => void;
}) {
  if (assets.length === 0) {
    return <p className="empty-state">No uploads yet — drop a file above to get started.</p>;
  }

  return (
    <div className="gallery-grid">
      {assets.map((asset) => {
        const thumb = asset.renditions.find(
          (r) => r.renditionType === 'THUMBNAIL' && r.isSelected,
        );
        return (
          <button key={asset.id} className="asset-card" onClick={() => onSelect(asset.id)}>
            <div className="asset-card-thumb">
              {thumb ? (
                <img src={mediaUrl(thumb.path)} alt={asset.originalFilename} />
              ) : (
                <div className="thumb-placeholder">{asset.kind === 'VIDEO' ? '\u{1F3AC}' : '\u{1F5BC}'}</div>
              )}
            </div>
            <div className="asset-card-info">
              <span className="asset-card-name" title={asset.originalFilename}>
                {asset.originalFilename}
              </span>
              <StatusBadge status={asset.status} />
            </div>
          </button>
        );
      })}
    </div>
  );
}
