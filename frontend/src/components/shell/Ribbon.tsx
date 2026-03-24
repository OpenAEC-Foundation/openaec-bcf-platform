import { useState, useRef, useEffect, useCallback } from 'react';
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
  const [prevTab, setPrevTab] = useState<TabId | null>(null);
  const [slideDirection, setSlideDirection] = useState<'left' | 'right'>('right');
  const tabsRef = useRef<HTMLDivElement>(null);
  const borderRef = useRef<HTMLDivElement>(null);
  const gapRef = useRef<HTMLDivElement>(null);

  const tabOrder: TabId[] = ['home', 'view'];

  const updateHighlight = useCallback(() => {
    const tabsEl = tabsRef.current;
    const borderEl = borderRef.current;
    const gapEl = gapRef.current;
    if (!tabsEl || !borderEl || !gapEl) return;

    // Find the active tab button (skip the file tab which is always first)
    const buttons = tabsEl.querySelectorAll<HTMLButtonElement>('.ribbon__tab:not(.ribbon__tab--file)');
    const activeIndex = tabOrder.indexOf(activeTab);
    const activeButton = buttons[activeIndex];

    if (!activeButton) return;

    const tabsRect = tabsEl.getBoundingClientRect();
    const btnRect = activeButton.getBoundingClientRect();

    const left = btnRect.left - tabsRect.left;
    const width = btnRect.width;
    const top = btnRect.top - tabsRect.top;
    const height = btnRect.height;

    borderEl.style.left = `${left}px`;
    borderEl.style.width = `${width}px`;
    borderEl.style.top = `${top}px`;
    borderEl.style.height = `${height}px`;
    borderEl.style.opacity = '1';

    gapEl.style.left = `${left + 1}px`;
    gapEl.style.width = `${width - 2}px`;
    gapEl.style.opacity = '1';
  }, [activeTab]);

  const switchTab = useCallback((newTab: TabId) => {
    if (newTab === activeTab) return;
    const oldIndex = tabOrder.indexOf(activeTab);
    const newIndex = tabOrder.indexOf(newTab);
    const direction = newIndex > oldIndex ? 'right' : 'left';
    setSlideDirection(direction);
    setPrevTab(activeTab);
    setActiveTab(newTab);

    // Clear the previous tab after exit animation
    setTimeout(() => setPrevTab(null), 250);
  }, [activeTab]);

  // Update highlight on tab change and resize
  useEffect(() => {
    updateHighlight();
    const handleResize = () => updateHighlight();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [updateHighlight]);

  const renderTabContent = (tabId: TabId) => {
    switch (tabId) {
      case 'home':
        return (
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
        );
      case 'view':
        return (
          <RibbonGroup label="Weergave">
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
        );
      default:
        return null;
    }
  };

  const enterClass = slideDirection === 'right' ? 'ribbon-panel-enter-right' : 'ribbon-panel-enter-left';
  const exitClass = slideDirection === 'right' ? 'ribbon-panel-exit-right' : 'ribbon-panel-exit-left';

  return (
    <div className="ribbon">
      {/* Tab strip */}
      <div className="ribbon__tabs" ref={tabsRef}>
        <button
          className="ribbon__tab ribbon__tab--file"
          onClick={onFileTabClick}
        >
          Bestand
        </button>
        <button
          className={`ribbon__tab ${activeTab === 'home' ? 'ribbon__tab--active' : ''}`}
          onClick={() => switchTab('home')}
        >
          Start
        </button>
        <button
          className={`ribbon__tab ${activeTab === 'view' ? 'ribbon__tab--active' : ''}`}
          onClick={() => switchTab('view')}
        >
          Beeld
        </button>
        {/* Animated border highlight */}
        <div className="ribbon-tab-border" ref={borderRef} />
        {/* Gap cover to connect tab to content */}
        <div className="ribbon-tab-gap" ref={gapRef} />
      </div>

      {/* Content panel */}
      <div className="ribbon__content">
        {/* Exit animation for previous tab */}
        {prevTab && (
          <div
            key={`exit-${prevTab}`}
            className={`ribbon__content-inner ${exitClass}`}
            style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'stretch', padding: '4px 8px', gap: '2px' }}
          >
            {renderTabContent(prevTab)}
          </div>
        )}
        {/* Enter animation for active tab */}
        <div
          key={`enter-${activeTab}`}
          className={`ribbon__content-inner ${prevTab ? enterClass : ''}`}
          style={{ display: 'flex', alignItems: 'stretch', flex: 1, gap: '2px' }}
        >
          {renderTabContent(activeTab)}
        </div>
      </div>
    </div>
  );
}
