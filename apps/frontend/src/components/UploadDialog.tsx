import { useCallback, useEffect, useRef, useState } from 'react';
import { CheckCircle2, CloudUpload, RotateCw, X, XCircle } from 'lucide-react';
import { uploadFile } from '../api';
import { formatBytes } from '../lib/format';
import type { MediaAsset } from '../types';
import { useToast } from './Toast';

const MAX_BYTES = 50 * 1024 * 1024;

type ItemState = 'uploading' | 'done' | 'error';
interface QueueItem {
  id: number;
  file: File;
  state: ItemState;
  progress: number;
  error?: string;
}

function validate(file: File): string | null {
  if (!file.type.startsWith('image/') && !file.type.startsWith('video/')) {
    return 'Only image and video files are supported';
  }
  if (file.size > MAX_BYTES) return `File is larger than ${formatBytes(MAX_BYTES)}`;
  return null;
}

export function UploadDialog({
  initialFiles,
  onUploaded,
  onClose,
}: {
  initialFiles: File[];
  onUploaded: (asset: MediaAsset) => void;
  onClose: () => void;
}) {
  const toast = useToast();
  const [items, setItems] = useState<QueueItem[]>([]);
  const [dragActive, setDragActive] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const nextId = useRef(1);

  const patch = useCallback((id: number, change: Partial<QueueItem>) => {
    setItems((prev) => prev.map((it) => (it.id === id ? { ...it, ...change } : it)));
  }, []);

  const start = useCallback(
    async (id: number, file: File) => {
      try {
        const asset = await uploadFile(file, (progress) => patch(id, { progress }));
        patch(id, { state: 'done', progress: 100 });
        onUploaded(asset);
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Upload failed';
        patch(id, { state: 'error', error: message });
        toast('error', `${file.name}: ${message}`);
      }
    },
    [onUploaded, patch, toast],
  );

  const addFiles = useCallback(
    (files: File[]) => {
      const added: QueueItem[] = files.map((file) => {
        const error = validate(file);
        return {
          id: nextId.current++,
          file,
          state: error ? 'error' : 'uploading',
          progress: 0,
          error: error ?? undefined,
        };
      });
      setItems((prev) => [...prev, ...added]);
      added.forEach((it) => {
        if (it.state === 'uploading') void start(it.id, it.file);
      });
    },
    [start],
  );

  // Files dropped on the page before the dialog opened.
  const seeded = useRef(false);
  useEffect(() => {
    if (seeded.current) return;
    seeded.current = true;
    if (initialFiles.length > 0) addFiles(initialFiles);
  }, [addFiles, initialFiles]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);

  function retry(item: QueueItem) {
    patch(item.id, { state: 'uploading', progress: 0, error: undefined });
    void start(item.id, item.file);
  }

  const busy = items.some((i) => i.state === 'uploading');

  return (
    <div className="modal-backdrop" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className="modal" role="dialog" aria-modal="true" aria-label="Upload media">
        <div className="modal-head">
          <h2>Upload media</h2>
          <button className="icon-button" onClick={onClose} aria-label="Close">
            <X size={18} />
          </button>
        </div>

        <div
          className={`dropzone ${dragActive ? 'active' : ''}`}
          onDragOver={(e) => {
            e.preventDefault();
            setDragActive(true);
          }}
          onDragLeave={() => setDragActive(false)}
          onDrop={(e) => {
            e.preventDefault();
            setDragActive(false);
            addFiles(Array.from(e.dataTransfer.files));
          }}
          onClick={() => inputRef.current?.click()}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              inputRef.current?.click();
            }
          }}
          role="button"
          tabIndex={0}
        >
          <input
            ref={inputRef}
            type="file"
            accept="image/*,video/*"
            multiple
            hidden
            onChange={(e) => {
              addFiles(Array.from(e.target.files ?? []));
              e.target.value = '';
            }}
          />
          <span className="empty-icon">
            <CloudUpload size={26} />
          </span>
          <p className="dropzone-title">Drag files here or click to browse</p>
          <p className="muted">Images and videos, up to {formatBytes(MAX_BYTES)} each</p>
        </div>

        {items.length > 0 && (
          <ul className="queue">
            {items.map((it) => (
              <li key={it.id} className={`queue-item ${it.state}`}>
                <div className="queue-main">
                  <span className="queue-name" title={it.file.name}>
                    {it.file.name}
                  </span>
                  <span className="muted">{formatBytes(it.file.size)}</span>
                </div>
                {it.state === 'uploading' && (
                  <div className="progress">
                    <div className="progress-bar" style={{ width: `${it.progress}%` }} />
                  </div>
                )}
                {it.state === 'done' && (
                  <span className="queue-status ok">
                    <CheckCircle2 size={16} /> Uploaded
                  </span>
                )}
                {it.state === 'error' && (
                  <span className="queue-status err">
                    <XCircle size={16} /> {it.error}
                    {validate(it.file) === null && (
                      <button className="icon-button small" onClick={() => retry(it)} aria-label="Retry">
                        <RotateCw size={14} />
                      </button>
                    )}
                  </span>
                )}
              </li>
            ))}
          </ul>
        )}

        <div className="modal-foot">
          <button className="btn btn-primary" onClick={onClose}>
            {busy ? 'Continue in background' : 'Done'}
          </button>
        </div>
      </div>
    </div>
  );
}
