import type { ListMediaResponse, MediaAsset, StatusUpdate } from './types';

export async function listAssets(): Promise<ListMediaResponse> {
  const res = await fetch('/api/assets');
  if (!res.ok) throw new Error(`listAssets failed: ${res.status}`);
  return res.json();
}

export async function getAsset(id: string): Promise<MediaAsset> {
  const res = await fetch(`/api/assets/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error(`getAsset failed: ${res.status}`);
  return res.json();
}

export async function uploadFile(
  file: File,
  onProgress?: (percent: number) => void,
): Promise<MediaAsset> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open('POST', '/api/uploads');

    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable && onProgress) {
        onProgress(Math.round((event.loaded / event.total) * 100));
      }
    };

    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        resolve(JSON.parse(xhr.responseText));
      } else {
        try {
          const body = JSON.parse(xhr.responseText);
          reject(new Error(body.message ?? body.error ?? `upload failed: ${xhr.status}`));
        } catch {
          reject(new Error(`upload failed: ${xhr.status}`));
        }
      }
    };
    xhr.onerror = () => reject(new Error('upload failed: network error'));

    const form = new FormData();
    form.append('file', file);
    xhr.send(form);
  });
}

/** URL the browser can fetch a rendition's image bytes from directly. */
export function mediaUrl(path: string): string {
  return `/media/${path}`;
}

export function streamAssetStatus(
  assetId: string,
  onUpdate: (update: StatusUpdate) => void,
): () => void {
  const source = new EventSource(`/api/events/asset/${encodeURIComponent(assetId)}`);
  source.onmessage = (event) => {
    onUpdate(JSON.parse(event.data));
  };
  return () => source.close();
}
