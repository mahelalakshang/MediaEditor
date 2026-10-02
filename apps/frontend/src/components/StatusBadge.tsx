import type { MediaStatus } from '../types';

const META: Record<MediaStatus, { label: string; tone: 'neutral' | 'info' | 'ok' | 'danger' }> = {
  UPLOADED: { label: 'Queued', tone: 'neutral' },
  PROCESSING: { label: 'Processing', tone: 'info' },
  PROCESSED: { label: 'Ready', tone: 'ok' },
  FAILED: { label: 'Failed', tone: 'danger' },
};

export function StatusBadge({ status }: { status: string }) {
  const meta = META[status as MediaStatus] ?? { label: status, tone: 'neutral' as const };
  return (
    <span className={`status-chip tone-${meta.tone}`}>
      <span className={`status-dot ${status === 'PROCESSING' ? 'pulse' : ''}`} />
      {meta.label}
    </span>
  );
}
