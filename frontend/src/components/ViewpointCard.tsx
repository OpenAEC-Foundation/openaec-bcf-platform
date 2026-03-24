import { Camera } from 'lucide-react';
import { viewpoints } from '../api/client';
import type { Viewpoint } from '../types/api';

interface Props {
  viewpoint: Viewpoint;
  projectId: string;
  topicId: string;
}

export default function ViewpointCard({ viewpoint, projectId, topicId }: Props) {
  const snapshotUrl = viewpoints.snapshotUrl(projectId, topicId, viewpoint.guid);

  return (
    <div className="bg-white rounded-lg border border-surface-dark overflow-hidden">
      {/* Snapshot preview */}
      <div className="aspect-video bg-surface-dark relative">
        <img
          src={snapshotUrl}
          alt="Viewpoint snapshot"
          className="w-full h-full object-cover"
          onError={(e) => {
            (e.target as HTMLImageElement).style.display = 'none';
            (e.target as HTMLImageElement).nextElementSibling?.classList.remove('hidden');
          }}
        />
        <div className="hidden absolute inset-0 flex items-center justify-center text-text-muted/40">
          <Camera size={32} />
        </div>
      </div>

      {/* Camera info */}
      {viewpoint.camera && (
        <div className="p-2.5 text-xs text-text-muted space-y-0.5">
          <p className="font-medium text-text">
            {viewpoint.camera.camera_type === 'orthogonal' ? 'Orthografisch' : 'Perspectief'}
          </p>
          <p>
            Positie: ({viewpoint.camera.x.toFixed(1)}, {viewpoint.camera.y.toFixed(1)}, {viewpoint.camera.z.toFixed(1)})
          </p>
        </div>
      )}
    </div>
  );
}
