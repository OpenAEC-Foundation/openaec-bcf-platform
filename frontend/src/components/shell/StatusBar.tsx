import { useState, useEffect } from 'react';
import './StatusBar.css';

interface StatusBarProps {
  projectCount?: number;
  topicCount?: number;
}

export default function StatusBar({ projectCount, topicCount }: StatusBarProps) {
  const [online, setOnline] = useState(navigator.onLine);

  useEffect(() => {
    const handleOnline = () => setOnline(true);
    const handleOffline = () => setOnline(false);
    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);
    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);

  return (
    <div className="status-bar">
      {projectCount !== undefined && (
        <div className="status-bar__item">
          <span className="status-bar__label">Projecten</span>
          <span className="status-bar__value">{projectCount}</span>
        </div>
      )}
      {topicCount !== undefined && (
        <>
          <div className="status-bar__separator" />
          <div className="status-bar__item">
            <span className="status-bar__label">Issues</span>
            <span className="status-bar__value">{topicCount}</span>
          </div>
        </>
      )}
      <div className="status-bar__spacer" />
      <div className="status-bar__item">
        <span className={`status-bar__dot ${online ? 'status-bar__dot--online' : 'status-bar__dot--offline'}`} />
        <span className="status-bar__value">{online ? 'Verbonden' : 'Offline'}</span>
      </div>
    </div>
  );
}
