import { useState } from 'react';
import RibbonTab from './RibbonTab';
import RibbonGroup from './RibbonGroup';
import RibbonButton from './RibbonButton';
import RibbonButtonStack from './RibbonButtonStack';
import { iconNewProject, iconImportBcf, iconExportBcf, iconNewIssue } from './icons';
import './Ribbon.css';

type TabId = 'home' | 'view';

interface RibbonProps {
  onFileTabClick: () => void;
  onNewProject?: () => void;
  onImportBcf?: () => void;
  onExportBcf?: () => void;
  onNewIssue?: () => void;
}

export default function Ribbon({
  onFileTabClick,
  onNewProject,
  onImportBcf,
  onExportBcf,
  onNewIssue,
}: RibbonProps) {
  const [activeTab, setActiveTab] = useState<TabId>('home');

  return (
    <div className="ribbon">
      {/* Tab strip */}
      <div className="ribbon__tabs">
        <RibbonTab
          label="Bestand"
          isFileTab
          onClick={onFileTabClick}
        />
        <RibbonTab
          label="Start"
          active={activeTab === 'home'}
          onClick={() => setActiveTab('home')}
        />
        <RibbonTab
          label="Beeld"
          active={activeTab === 'view'}
          onClick={() => setActiveTab('view')}
        />
      </div>

      {/* Content panel */}
      <div className="ribbon__content">
        {activeTab === 'home' && (
          <>
            <RibbonGroup label="Project">
              <RibbonButton
                icon={iconNewProject}
                label="Nieuw project"
                onClick={onNewProject}
              />
              <RibbonButtonStack>
                <RibbonButton
                  icon={iconImportBcf}
                  label="BCF importeren"
                  onClick={onImportBcf}
                  small
                />
                <RibbonButton
                  icon={iconExportBcf}
                  label="BCF exporteren"
                  onClick={onExportBcf}
                  small
                />
              </RibbonButtonStack>
            </RibbonGroup>
            <RibbonGroup label="Issues">
              <RibbonButton
                icon={iconNewIssue}
                label="Nieuw issue"
                onClick={onNewIssue}
              />
            </RibbonGroup>
            <RibbonGroup label="Acties">
              <RibbonButton
                icon={iconExportBcf}
                label="Exporteren"
                onClick={onExportBcf}
              />
            </RibbonGroup>
          </>
        )}
        {activeTab === 'view' && (
          <RibbonGroup label="Weergave">
            {/* Placeholder for future view options */}
            <div style={{
              display: 'flex',
              alignItems: 'center',
              padding: '0 16px',
              fontSize: '11px',
              color: 'var(--theme-text-muted)',
            }}>
              Weergave-opties komen hier
            </div>
          </RibbonGroup>
        )}
      </div>
    </div>
  );
}
