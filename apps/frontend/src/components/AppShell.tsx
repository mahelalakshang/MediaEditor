import type { ReactNode } from 'react';
import { Layers, Moon, Sun, Upload } from 'lucide-react';
import type { Theme } from '../hooks/useTheme';

export function AppShell({
  theme,
  onToggleTheme,
  onUpload,
  onHome,
  children,
}: {
  theme: Theme;
  onToggleTheme: () => void;
  onUpload: () => void;
  onHome: () => void;
  children: ReactNode;
}) {
  return (
    <div className="shell">
      <header className="topbar">
        <div className="topbar-inner">
          <button className="brand" onClick={onHome} aria-label="Go to library">
            <span className="brand-mark">
              <Layers size={18} />
            </span>
            <span className="brand-name">Media Studio</span>
          </button>
          <div className="topbar-actions">
            <button
              className="icon-button"
              onClick={onToggleTheme}
              aria-label={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
              title="Toggle theme"
            >
              {theme === 'dark' ? <Sun size={18} /> : <Moon size={18} />}
            </button>
            <button className="btn btn-primary" onClick={onUpload}>
              <Upload size={16} />
              Upload
            </button>
          </div>
        </div>
      </header>
      <main className="content">{children}</main>
    </div>
  );
}
