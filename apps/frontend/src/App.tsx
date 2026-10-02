import { useCallback, useEffect, useRef, useState } from 'react';
import { CloudUpload } from 'lucide-react';
import { listAssets } from './api';
import type { MediaAsset } from './types';
import { useHashRoute } from './hooks/useHashRoute';
import { useTheme } from './hooks/useTheme';
import { AppShell } from './components/AppShell';
import { AssetDetail } from './components/AssetDetail';
import { Library } from './components/Library';
import { ToastProvider, useToast } from './components/Toast';
import { UploadDialog } from './components/UploadDialog';
import './styles/tokens.css';
import './styles/layout.css';
import './styles/components.css';

const POLL_MS = 3000;

function AppInner() {
  const { theme, toggle } = useTheme();
  const { route, navigate } = useHashRoute();
  const toast = useToast();

  const [assets, setAssets] = useState<MediaAsset[]>([]);
  const [loading, setLoading] = useState(true);
  const [uploadOpen, setUploadOpen] = useState(false);
  const [droppedFiles, setDroppedFiles] = useState<File[]>([]);
  const [pageDrag, setPageDrag] = useState(false);
  const dragDepth = useRef(0);

  const refresh = useCallback(() => {
    listAssets()
      .then((res) => setAssets(res.assets))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  // Keep in-flight items fresh on the library without a manual reload.
  const hasActive = assets.some((a) => a.status === 'UPLOADED' || a.status === 'PROCESSING');
  useEffect(() => {
    if (route.name !== 'library' || !hasActive) return;
    const timer = window.setInterval(refresh, POLL_MS);
    return () => window.clearInterval(timer);
  }, [route.name, hasActive, refresh]);

  const handleUploaded = useCallback(
    (asset: MediaAsset) => {
      setAssets((prev) => [asset, ...prev.filter((a) => a.id !== asset.id)]);
      toast('success', `${asset.originalFilename} uploaded`);
    },
    [toast],
  );

  function openUpload(files: File[] = []) {
    setDroppedFiles(files);
    setUploadOpen(true);
  }

  // Drop files anywhere on the page to start an upload.
  useEffect(() => {
    const hasFiles = (e: DragEvent) => Array.from(e.dataTransfer?.types ?? []).includes('Files');
    const enter = (e: DragEvent) => {
      if (!hasFiles(e)) return;
      dragDepth.current++;
      setPageDrag(true);
    };
    const leave = (e: DragEvent) => {
      if (!hasFiles(e)) return;
      dragDepth.current = Math.max(0, dragDepth.current - 1);
      if (dragDepth.current === 0) setPageDrag(false);
    };
    const over = (e: DragEvent) => {
      if (hasFiles(e)) e.preventDefault();
    };
    const drop = (e: DragEvent) => {
      if (!hasFiles(e)) return;
      e.preventDefault();
      dragDepth.current = 0;
      setPageDrag(false);
      // The dialog has its own drop target; only handle drops elsewhere.
      if ((e.target as HTMLElement).closest('.modal-backdrop')) return;
      const files = Array.from(e.dataTransfer?.files ?? []);
      if (files.length > 0) openUpload(files);
    };
    window.addEventListener('dragenter', enter);
    window.addEventListener('dragleave', leave);
    window.addEventListener('dragover', over);
    window.addEventListener('drop', drop);
    return () => {
      window.removeEventListener('dragenter', enter);
      window.removeEventListener('dragleave', leave);
      window.removeEventListener('dragover', over);
      window.removeEventListener('drop', drop);
    };
  }, []);

  return (
    <AppShell
      theme={theme}
      onToggleTheme={toggle}
      onUpload={() => openUpload()}
      onHome={() => navigate({ name: 'library' })}
    >
      {route.name === 'asset' ? (
        <AssetDetail
          assetId={route.id}
          onBack={() => {
            navigate({ name: 'library' });
            refresh();
          }}
        />
      ) : (
        <Library
          assets={assets}
          loading={loading}
          onSelect={(id) => navigate({ name: 'asset', id })}
          onUpload={() => openUpload()}
        />
      )}

      {uploadOpen && (
        <UploadDialog
          initialFiles={droppedFiles}
          onUploaded={handleUploaded}
          onClose={() => setUploadOpen(false)}
        />
      )}

      {pageDrag && !uploadOpen && (
        <div className="drop-overlay" aria-hidden>
          <CloudUpload size={40} />
          <p>Drop to upload</p>
        </div>
      )}
    </AppShell>
  );
}

export default function App() {
  return (
    <ToastProvider>
      <AppInner />
    </ToastProvider>
  );
}
