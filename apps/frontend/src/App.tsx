import { useCallback, useEffect, useState } from 'react';
import { listAssets } from './api';
import type { MediaAsset } from './types';
import { UploadDropzone } from './components/UploadDropzone';
import { Gallery } from './components/Gallery';
import { AssetDetail } from './components/AssetDetail';
import './styles.css';

export default function App() {
  const [assets, setAssets] = useState<MediaAsset[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(() => {
    listAssets()
      .then((res) => setAssets(res.assets))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  function handleUploaded(asset: MediaAsset) {
    setAssets((prev) => [asset, ...prev]);
    setSelectedId(asset.id);
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Media Pipeline</h1>
        <p className="subtitle">Rust · Axum · gRPC · Kafka · ffmpeg · NestJS BFF · React</p>
      </header>

      <UploadDropzone onUploaded={handleUploaded} />

      {selectedId ? (
        <AssetDetail
          assetId={selectedId}
          onBack={() => {
            setSelectedId(null);
            refresh();
          }}
        />
      ) : loading ? (
        <p className="empty-state">Loading…</p>
      ) : (
        <Gallery assets={assets} onSelect={setSelectedId} />
      )}
    </div>
  );
}
