import { useState, useEffect, useCallback } from 'react';
import { iconImportBcf, iconExportBcf, iconSettings, iconAbout, iconClose } from './icons';
import './Backstage.css';

type Panel = 'none' | 'about';

interface BackstageProps {
  open: boolean;
  onClose: () => void;
  onImportBcf?: () => void;
  onExportBcf?: () => void;
}

export default function Backstage({ open, onClose, onImportBcf, onExportBcf }: BackstageProps) {
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

  if (!open) return null;

  return (
    <div className="backstage">
      <div className="backstage__sidebar">
        <div className="backstage__sidebar-header">Bestand</div>

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

        <div className="backstage__separator" />

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

        <div className="backstage__separator" />

        <button className="backstage__item" onClick={onClose}>
          <span className="backstage__item-icon" dangerouslySetInnerHTML={{ __html: iconClose }} />
          <span className="backstage__item-label">Sluiten</span>
          <span className="backstage__item-shortcut">Esc</span>
        </button>
      </div>

      <div className="backstage__panel">
        {activePanel === 'about' && (
          <div>
            <div className="backstage__panel-title">Over OpenAEC BCF Platform</div>
            <div className="backstage__panel-text">
              Een centraal BCF issue management platform voor BIM-projecten.
              <br />
              Importeer en exporteer .bcfzip bestanden, beheer issues en viewpoints,
              en werk samen met je team.
            </div>
            <div className="backstage__panel-version">
              Versie 0.1.0 &middot; BCF 2.1 compatibel
            </div>
          </div>
        )}
        {activePanel === 'none' && (
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#A1A1AA', fontSize: '13px' }}>
            Kies een optie in het menu
          </div>
        )}
      </div>
    </div>
  );
}
