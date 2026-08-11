import type { GrpcMediaAsset, GrpcMediaStatusUpdate, GrpcRendition } from './media-grpc.service';

function stripEnumPrefix(value: string | undefined, prefix: string): string {
  if (!value) return '';
  return value.startsWith(prefix) ? value.slice(prefix.length) : value;
}

export function toRenditionDto(r: GrpcRendition) {
  return {
    id: r.id,
    renditionType: stripEnumPrefix(r.renditionType, 'RENDITION_TYPE_'),
    path: r.path,
    width: r.width,
    height: r.height,
    score: r.score ?? null,
    isSelected: r.isSelected,
    timestampSeconds: r.timestampSeconds ?? null,
  };
}

export function toAssetDto(a: GrpcMediaAsset) {
  return {
    id: a.id,
    originalFilename: a.originalFilename,
    mimeType: a.mimeType,
    kind: stripEnumPrefix(a.kind, 'MEDIA_KIND_'),
    status: stripEnumPrefix(a.status, 'MEDIA_STATUS_'),
    createdAt: a.createdAt,
    renditions: (a.renditions ?? []).map(toRenditionDto),
  };
}

export function toStatusUpdateDto(u: GrpcMediaStatusUpdate) {
  return {
    assetId: u.assetId,
    status: stripEnumPrefix(u.status, 'MEDIA_STATUS_'),
    message: u.message ?? null,
    updatedAtMs: u.updatedAtMs,
    progressHint: u.progressHint ?? null,
  };
}
