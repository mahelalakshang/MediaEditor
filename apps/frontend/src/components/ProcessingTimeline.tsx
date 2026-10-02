import { Check, Loader2, X } from 'lucide-react';
import type { MediaStatus } from '../types';

type StepState = 'done' | 'active' | 'pending' | 'failed';

function steps(status: MediaStatus): { label: string; state: StepState }[] {
  const failed = status === 'FAILED';
  return [
    { label: 'Uploaded', state: 'done' },
    {
      label: 'Processing',
      state: status === 'UPLOADED' ? 'pending' : status === 'PROCESSING' ? 'active' : 'done',
    },
    {
      label: failed ? 'Failed' : 'Ready',
      state: failed ? 'failed' : status === 'PROCESSED' ? 'done' : 'pending',
    },
  ];
}

export function ProcessingTimeline({
  status,
  message,
}: {
  status: MediaStatus;
  message?: string | null;
}) {
  return (
    <div>
      <ol className="timeline">
        {steps(status).map((s) => (
          <li key={s.label} className={`step ${s.state}`}>
            <span className="step-marker">
              {s.state === 'done' && <Check size={13} />}
              {s.state === 'active' && <Loader2 size={13} className="spin" />}
              {s.state === 'failed' && <X size={13} />}
            </span>
            <span className="step-label">{s.label}</span>
          </li>
        ))}
      </ol>
      {message && <p className="muted timeline-message">{message}</p>}
    </div>
  );
}
