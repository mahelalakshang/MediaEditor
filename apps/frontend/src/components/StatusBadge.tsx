const COLORS: Record<string, string> = {
  UPLOADED: '#64748b',
  PROCESSING: '#d97706',
  PROCESSED: '#16a34a',
  FAILED: '#dc2626',
};

export function StatusBadge({ status }: { status: string }) {
  return (
    <span className="status-badge" style={{ backgroundColor: COLORS[status] ?? '#64748b' }}>
      {status}
    </span>
  );
}
