import { useState, useEffect, useCallback } from 'react';
import { brand } from '../../config/brand';
import { iconImportBcf, iconExportBcf, iconSettings, iconAbout, iconCloud } from './icons';
import CloudPanel from './CloudPanel';
import './Backstage.css';
import './CloudPanel.css';

type Panel = 'none' | 'about' | 'cloud';

interface BackstageProps {
  open: boolean;
  onClose: () => void;
  onImportBcf?: () => void;
  onExportBcf?: () => void;
  onImportFromCloud?: (file: File) => void;
  projectId?: string;
  projectName?: string;
}

const iconBack = `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>`;

export default function Backstage({ open, onClose, onImportBcf, onExportBcf, onImportFromCloud, projectId, projectName }: BackstageProps) {
  const [activePanel, setActivePanel] = useState<Panel>('none');

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    },
    [onClose],
  );

  useEffect(() => {
    if (open) {
      document.addEventListener('keydown', handleKeyDown);
      return () => document.removeEventListener('keydown', handleKeyDown);
    }
  }, [open, handleKeyDown]);

  // Reset active panel when closing
  useEffect(() => {
    if (!open) setActivePanel('none');
  }, [open]);

  if (!open) return null;

  return (
    <div className="backstage">
      <div className="backstage__sidebar">
        {/* Back button */}
        <button className="backstage__back-btn" onClick={onClose}>
          <span dangerouslySetInnerHTML={{ __html: iconBack }} />
          <span>Terug</span>
        </button>

        {/* Menu items */}
        <button
          className="backstage__item"
          onClick={() => {
            onImportBcf?.();
            onClose();
          }}
        >
          <span className="backstage__item-icon" dangerouslySetInnerHTML={{ __html: iconImportBcf }} />
          <span className="backstage__item-label">BCF importeren</span>
          <span className="backstage__item-shortcut">Ctrl+I</span>
        </button>

        <button
          className="backstage__item"
          onClick={() => {
            onExportBcf?.();
            onClose();
          }}
        >
          <span className="backstage__item-icon" dangerouslySetInnerHTML={{ __html: iconExportBcf }} />
          <span className="backstage__item-label">BCF exporteren</span>
          <span className="backstage__item-shortcut">Ctrl+E</span>
        </button>

        <button
          className={`backstage__item ${activePanel === 'cloud' ? 'backstage__item--active' : ''}`}
          onClick={() => setActivePanel(activePanel === 'cloud' ? 'none' : 'cloud')}
        >
          <span className="backstage__item-icon" dangerouslySetInnerHTML={{ __html: iconCloud }} />
          <span className="backstage__item-label">Cloud opslag</span>
        </button>

        <div className="backstage__divider" />

        <button className="backstage__item backstage__item--disabled">
          <span className="backstage__item-icon" dangerouslySetInnerHTML={{ __html: iconSettings }} />
          <span className="backstage__item-label">Voorkeuren</span>
        </button>

        <button
          className={`backstage__item ${activePanel === 'about' ? 'backstage__item--active' : ''}`}
          onClick={() => setActivePanel(activePanel === 'about' ? 'none' : 'about')}
        >
          <span className="backstage__item-icon" dangerouslySetInnerHTML={{ __html: iconAbout }} />
          <span className="backstage__item-label">Over</span>
        </button>

        <div className="backstage__divider" />

        <button className="backstage__item" onClick={onClose}>
          <span className="backstage__item-label">Sluiten</span>
          <span className="backstage__item-shortcut">Esc</span>
        </button>
      </div>

      <div className="backstage__panel">
        {activePanel === 'cloud' && (
          <CloudPanel
            projectId={projectId}
            projectName={projectName}
            onImportFromCloud={onImportFromCloud}
            onClose={onClose}
          />
        )}
        {activePanel === 'about' && (
          <div className="backstage__about">
            <div className="backstage__about-brand">
              <span className="backstage__about-brand-prefix">{brand.namePrefix}</span>
              <span className="backstage__about-brand-accent">{brand.nameAccent}</span>
            </div>
            <div className="backstage__about-product">{brand.product}</div>
            <div className="backstage__about-tagline">{brand.tagline}</div>
            <div className="backstage__about-separator" />
            <div className="backstage__about-text">
              Een centraal BCF issue management platform voor BIM-projecten.
              Importeer en exporteer .bcfzip bestanden, beheer issues en viewpoints,
              en werk samen met je team.
            </div>
            <div className="backstage__about-version">
              Versie 0.1.0 &middot; BCF 2.1 compatibel
            </div>
          </div>
        )}
        {activePanel === 'none' && (
          <div className="backstage__panel-placeholder">
            Kies een optie in het menu
          </div>
        )}
      </div>
    </div>
  );
}
