import { useEffect, useState } from 'react';
import { getAsset, mediaUrl, streamAssetStatus } from '../api';
import type { MediaAsset, StatusUpdate } from '../types';
import { StatusBadge } from './StatusBadge';

export function AssetDetail({ assetId, onBack }: { assetId: string; onBack: () => void }) {
  const [asset, setAsset] = useState<MediaAsset | null>(null);
  const [liveStatus, setLiveStatus] = useState<StatusUpdate | null>(null);

  useEffect(() => {
    setAsset(null);
    setLiveStatus(null);
    let cancelled = false;
    getAsset(assetId).then((a) => {
      if (!cancelled) setAsset(a);
    });
    return () => {
      cancelled = true;
    };
  }, [assetId]);

  useEffect(() => {
    const stop = streamAssetStatus(assetId, (update) => {
      setLiveStatus(update);
      if (update.status === 'PROCESSED' || update.status === 'FAILED') {
        getAsset(assetId).then(setAsset);
      }
    });
    return stop;
  }, [assetId]);

  if (!asset) {
    return (
      <div className="panel">
        <button className="link-button" onClick={onBack}>
          &larr; back to gallery
        </button>
        <p className="empty-state">Loading…</p>
      </div>
    );
  }

  const status = liveStatus?.status ?? asset.status;
  const thumbnail = asset.renditions.find((r) => r.renditionType === 'THUMBNAIL' && r.isSelected);
  const candidates = asset.renditions
    .filter((r) => r.renditionType === 'CANDIDATE_FRAME')
    .sort((a, b) => (a.timestampSeconds ?? 0) - (b.timestampSeconds ?? 0));

  return (
    <div className="panel asset-detail">
      <button className="link-button" onClick={onBack}>
        &larr; back to gallery
      </button>

      <div className="asset-detail-header">
        <h2>{asset.originalFilename}</h2>
        <StatusBadge status={status} />
      </div>
      {liveStatus?.message && <p className="status-message">{liveStatus.message}</p>}

      {thumbnail ? (
        <img className="hero-thumbnail" src={mediaUrl(thumbnail.path)} alt={asset.originalFilename} />
      ) : (
        <div className="hero-thumbnail placeholder">
          {status === 'FAILED' ? 'processing failed' : 'processing…'}
        </div>
      )}

      {candidates.length > 0 && (
        <>
          <h3>Candidate frames (smart thumbnail scoring)</h3>
          <p className="hint">
            Every sampled frame is scored for sharpness, contrast, and brightness balance — the
            highest-scoring frame becomes the thumbnail above.
          </p>
          <div className="candidates-grid">
            {candidates.map((c) => (
              <div key={c.id} className={`candidate ${c.isSelected ? 'selected' : ''}`}>
                <img src={mediaUrl(c.path)} alt={`candidate frame at ${c.timestampSeconds}s`} />
                <div className="candidate-meta">
                  <span>{c.timestampSeconds?.toFixed(1)}s</span>
                  <span>score {c.score?.toFixed(3)}</span>
                  {c.isSelected && <span className="badge-selected">selected</span>}
                </div>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
