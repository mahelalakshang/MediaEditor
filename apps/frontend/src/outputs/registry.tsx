import type { ComponentType } from 'react';
import { Clapperboard, Download, Image as ImageIcon, Images } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { mediaUrl } from '../api';
import { baseName } from '../lib/format';
import type { MediaAsset, Rendition, RenditionType } from '../types';
import { CandidateFrames } from './CandidateFrames';

export interface OutputViewerProps {
  asset: MediaAsset;
  renditions: Rendition[];
}

export interface OutputDefinition {
  label: string;
  icon: LucideIcon;
  Viewer: ComponentType<OutputViewerProps>;
}

function ThumbnailViewer({ asset, renditions }: OutputViewerProps) {
  const thumb = renditions.find((r) => r.isSelected) ?? renditions[0];
  if (!thumb) return null;
  return (
    <div className="output-thumb">
      <img src={mediaUrl(thumb.path)} alt={`Thumbnail of ${asset.originalFilename}`} />
      <div className="output-thumb-side">
        <dl className="kv">
          <dt>Size</dt>
          <dd>
            {thumb.width} × {thumb.height}
          </dd>
          <dt>Format</dt>
          <dd>JPEG</dd>
          {thumb.timestampSeconds != null && (
            <>
              <dt>Taken at</dt>
              <dd>{thumb.timestampSeconds.toFixed(1)}s</dd>
            </>
          )}
          {thumb.score != null && (
            <>
              <dt>Score</dt>
              <dd>{thumb.score.toFixed(3)}</dd>
            </>
          )}
        </dl>
        <a
          className="btn"
          href={mediaUrl(thumb.path)}
          download={`${baseName(asset.originalFilename)}_thumbnail.jpg`}
        >
          <Download size={16} />
          Download thumbnail
        </a>
      </div>
    </div>
  );
}

function PreviewClipViewer({ renditions }: OutputViewerProps) {
  const clip = renditions[0];
  if (!clip) return null;
  return <video className="output-video" src={mediaUrl(clip.path)} controls preload="metadata" />;
}

/**
 * One entry per processor output. To surface a new processor in the UI, add its
 * RenditionType here (order = display order); AssetDetail renders the rest.
 */
export const OUTPUTS: Partial<Record<RenditionType, OutputDefinition>> = {
  THUMBNAIL: { label: 'Thumbnail', icon: ImageIcon, Viewer: ThumbnailViewer },
  CANDIDATE_FRAME: { label: 'Candidate frames', icon: Images, Viewer: CandidateFrames },
  PREVIEW_CLIP: { label: 'Preview clip', icon: Clapperboard, Viewer: PreviewClipViewer },
};

export const OUTPUT_ORDER = Object.keys(OUTPUTS) as RenditionType[];
