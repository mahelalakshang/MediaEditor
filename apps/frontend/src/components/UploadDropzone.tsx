import { useRef, useState } from 'react';
import { uploadFile } from '../api';
import type { MediaAsset } from '../types';

export function UploadDropzone({ onUploaded }: { onUploaded: (asset: MediaAsset) => void }) {
  const [dragActive, setDragActive] = useState(false);
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  async function handleFile(file: File) {
    setError(null);
    setProgress(0);
    try {
      const asset = await uploadFile(file, setProgress);
      onUploaded(asset);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'upload failed');
    } finally {
      setProgress(null);
    }
  }

  return (
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
        const file = e.dataTransfer.files[0];
        if (file) void handleFile(file);
      }}
      onClick={() => inputRef.current?.click()}
      role="button"
      tabIndex={0}
    >
      <input
        ref={inputRef}
        type="file"
        accept="image/*,video/*"
        hidden
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) void handleFile(file);
          e.target.value = '';
        }}
      />
      {progress !== null ? (
        <div className="upload-progress">
          <div className="upload-progress-track">
            <div className="upload-progress-bar" style={{ width: `${progress}%` }} />
          </div>
          <span>{progress}%</span>
        </div>
      ) : (
        <p>Drag &amp; drop an image or video, or click to choose a file</p>
      )}
      {error && <p className="error-text">{error}</p>}
    </div>
  );
}
