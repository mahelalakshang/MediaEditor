import { useEffect, useState } from 'react';
import { ArrowLeft, Check, Copy, Download, FileQuestion } from 'lucide-react';
import { getAsset, mediaUrl, streamAssetStatus } from '../api';
import { fullDate } from '../lib/format';
import { OUTPUTS, OUTPUT_ORDER } from '../outputs/registry';
import type { MediaAsset, StatusUpdate } from '../types';
import { ProcessingTimeline } from './ProcessingTimeline';
import { StatusBadge } from './StatusBadge';
import { useToast } from './Toast';

export function AssetDetail({ assetId, onBack }: { assetId: string; onBack: () => void }) {
  const toast = useToast();
  const [asset, setAsset] = useState<MediaAsset | null>(null);
  const [liveStatus, setLiveStatus] = useState<StatusUpdate | null>(null);
  const [loadError, setLoadError] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    setAsset(null);
    setLiveStatus(null);
    setLoadError(false);
    let cancelled = false;
    getAsset(assetId)
      .then((a) => {
        if (!cancelled) setAsset(a);
      })
      .catch(() => {
        if (!cancelled) setLoadError(true);
      });
    return () => {
      cancelled = true;
    };
  }, [assetId]);

  useEffect(() => {
    const stop = streamAssetStatus(assetId, (update) => {
      setLiveStatus(update);
      if (update.status === 'PROCESSED' || update.status === 'FAILED') {
        getAsset(assetId).then(setAsset).catch(() => {});
      }
    });
    return stop;
  }, [assetId]);

  const back = (
    <button className="back-link" onClick={onBack}>
      <ArrowLeft size={16} />
      Library
    </button>
  );

  if (loadError) {
    return (
      <section>
        {back}
        <div className="empty">
          <span className="empty-icon">
            <FileQuestion size={28} />
          </span>
          <h2>Asset not found</h2>
          <p className="muted">It may have been removed, or the link is wrong.</p>
        </div>
      </section>
    );
  }

  if (!asset) {
    return (
      <section>
        {back}
        <div className="detail-grid" aria-hidden>
          <div className="skeleton preview-skeleton" />
          <div className="skeleton side-skeleton" />
        </div>
      </section>
    );
  }

  const status = liveStatus?.status ?? asset.status;
  const thumbnail = asset.renditions.find((r) => r.renditionType === 'THUMBNAIL' && r.isSelected);
  const unknown = asset.renditions.filter((r) => !(r.renditionType in OUTPUTS));
  const outputs = OUTPUT_ORDER.map((type) => ({
    type,
    def: OUTPUTS[type]!,
    renditions: asset.renditions.filter((r) => r.renditionType === type),
  })).filter((o) => o.renditions.length > 0);

  async function copyId() {
    try {
      await navigator.clipboard.writeText(asset!.id);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      toast('error', 'Could not copy to clipboard');
    }
  }

  return (
    <section>
      {back}

      <div className="detail-head">
        <h1 title={asset.originalFilename}>{asset.originalFilename}</h1>
        <StatusBadge status={status} />
      </div>

      <div className="detail-grid">
        <div className="preview">
          {asset.kind === 'VIDEO' ? (
            <video
              src={mediaUrl(asset.originalPath)}
              poster={thumbnail ? mediaUrl(thumbnail.path) : undefined}
              controls
              preload="metadata"
            />
          ) : (
            <img src={mediaUrl(asset.originalPath)} alt={asset.originalFilename} />
          )}
        </div>

        <aside className="side">
          <div className="card">
            <h3>Details</h3>
            <dl className="kv">
              <dt>Type</dt>
              <dd>{asset.mimeType}</dd>
              <dt>Kind</dt>
              <dd>{asset.kind === 'VIDEO' ? 'Video' : 'Image'}</dd>
              <dt>Uploaded</dt>
              <dd>{fullDate(asset.createdAt)}</dd>
              <dt>ID</dt>
              <dd className="id-row">
                <code title={asset.id}>{asset.id.slice(0, 8)}…</code>
                <button className="icon-button small" onClick={copyId} aria-label="Copy ID">
                  {copied ? <Check size={14} /> : <Copy size={14} />}
                </button>
              </dd>
            </dl>
            <a
              className="btn btn-block"
              href={mediaUrl(asset.originalPath)}
              download={asset.originalFilename}
            >
              <Download size={16} />
              Download original
            </a>
          </div>

          <div className="card">
            <h3>Processing</h3>
            <ProcessingTimeline status={status} message={liveStatus?.message} />
          </div>
        </aside>
      </div>

      {outputs.length > 0 && <h2 className="section-title">Outputs</h2>}
      <div className="outputs">
        {outputs.map(({ type, def, renditions }) => (
          <div key={type} className="card output-card">
            <h3>
              <def.icon size={16} />
              {def.label}
              <span className="count">{renditions.length}</span>
            </h3>
            <def.Viewer asset={asset} renditions={renditions} />
          </div>
        ))}
        {unknown.length > 0 && (
          <div className="card output-card">
            <h3>Other files</h3>
            <ul className="file-list">
              {unknown.map((r) => (
                <li key={r.id}>
                  <span>{r.renditionType}</span>
                  <a className="btn btn-sm" href={mediaUrl(r.path)} download>
                    <Download size={14} />
                    Download
                  </a>
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </section>
  );
}
