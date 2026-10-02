import { Download } from 'lucide-react';
import { mediaUrl } from '../api';
import { baseName } from '../lib/format';
import type { OutputViewerProps } from './registry';

export function CandidateFrames({ asset, renditions }: OutputViewerProps) {
  const frames = [...renditions].sort(
    (a, b) => (a.timestampSeconds ?? 0) - (b.timestampSeconds ?? 0),
  );
  return (
    <>
      <p className="muted output-hint">
        Frames sampled across the video are scored for sharpness, contrast and brightness balance.
        The highest score becomes the thumbnail.
      </p>
      <div className="candidates-grid">
        {frames.map((c, i) => (
          <figure key={c.id} className={`candidate ${c.isSelected ? 'selected' : ''}`}>
            <img src={mediaUrl(c.path)} alt={`Frame at ${c.timestampSeconds ?? 0}s`} loading="lazy" />
            <figcaption>
              <span>{(c.timestampSeconds ?? 0).toFixed(1)}s</span>
              <span className="muted">score {(c.score ?? 0).toFixed(3)}</span>
              {c.isSelected && <span className="pill-selected">Selected</span>}
              <a
                className="mini-download"
                href={mediaUrl(c.path)}
                download={`${baseName(asset.originalFilename)}_frame_${i + 1}.jpg`}
                aria-label={`Download frame ${i + 1}`}
                title="Download frame"
              >
                <Download size={13} />
              </a>
            </figcaption>
          </figure>
        ))}
      </div>
    </>
  );
}
