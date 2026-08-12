export type MediaKind = 'IMAGE' | 'VIDEO';
export type MediaStatus = 'UPLOADED' | 'PROCESSING' | 'PROCESSED' | 'FAILED';
export type RenditionType = 'THUMBNAIL' | 'CANDIDATE_FRAME' | 'PREVIEW_CLIP';

export interface Rendition {
  id: string;
  renditionType: RenditionType;
  path: string;
  width: number;
  height: number;
  score: number | null;
  isSelected: boolean;
  timestampSeconds: number | null;
}

export interface MediaAsset {
  id: string;
  originalFilename: string;
  mimeType: string;
  kind: MediaKind;
  status: MediaStatus;
  createdAt: string;
  renditions: Rendition[];
  originalPath: string;
}

export interface ListMediaResponse {
  assets: MediaAsset[];
  nextPageToken: string;
}

export interface StatusUpdate {
  assetId: string;
  status: MediaStatus;
  message: string | null;
  updatedAtMs: string;
  progressHint: string | null;
}
