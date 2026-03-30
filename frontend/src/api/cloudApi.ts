// Cloud storage API client — Nextcloud WebDAV proxy

import { ApiError } from './client';
import type {
  CloudStatus,
  CloudProject,
  CloudFile,
  CloudUploadResponse,
  CloudSaveResponse,
} from '../types/api';

const BASE = '';

function authHeaders(): HeadersInit {
  const headers: HeadersInit = {};
  const token = localStorage.getItem('bcf_token');
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  return headers;
}

async function request<T>(url: string, opts?: RequestInit): Promise<T> {
  const res = await fetch(BASE + url, {
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    ...opts,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new ApiError(res.status, body.error || res.statusText);
  }
  return res.json();
}

/** Check if cloud storage is enabled and connected. */
export function cloudStatus(): Promise<CloudStatus> {
  return request<CloudStatus>('/api/cloud/status');
}

/** List project folders on Nextcloud. */
export async function cloudListProjects(): Promise<CloudProject[]> {
  const res = await request<{ projects: CloudProject[] }>('/api/cloud/projects');
  return res.projects;
}

/** List files in a project's bcf-platform subfolder. */
export async function cloudListFiles(project: string): Promise<CloudFile[]> {
  const encoded = encodeURIComponent(project);
  const res = await request<{ files: CloudFile[] }>(`/api/cloud/projects/${encoded}/files`);
  return res.files;
}

/** Download a file from cloud storage. */
export async function cloudDownloadFile(project: string, filename: string): Promise<Blob> {
  const url = `${BASE}/api/cloud/projects/${encodeURIComponent(project)}/files/${encodeURIComponent(filename)}`;
  const res = await fetch(url, { headers: authHeaders() });
  if (!res.ok) {
    throw new ApiError(res.status, `Download failed: ${res.statusText}`);
  }
  return res.blob();
}

/** Upload a file to cloud storage. */
export async function cloudUploadFile(
  project: string,
  filename: string,
  file: File | Blob,
): Promise<CloudUploadResponse> {
  const url = `${BASE}/api/cloud/projects/${encodeURIComponent(project)}/files/${encodeURIComponent(filename)}`;
  const form = new FormData();
  form.append('file', file);
  const res = await fetch(url, {
    method: 'PUT',
    headers: authHeaders(),
    body: form,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new ApiError(res.status, body.error || res.statusText);
  }
  return res.json();
}

/** Delete a file from cloud storage. */
export async function cloudDeleteFile(project: string, filename: string): Promise<void> {
  const url = `${BASE}/api/cloud/projects/${encodeURIComponent(project)}/files/${encodeURIComponent(filename)}`;
  const res = await fetch(url, {
    method: 'DELETE',
    headers: authHeaders(),
  });
  if (!res.ok) {
    throw new ApiError(res.status, `Delete failed: ${res.statusText}`);
  }
}

/** Export a BCF project from the database and save it to Nextcloud. */
export function cloudSaveBcf(
  cloudProject: string,
  projectId: string,
): Promise<CloudSaveResponse> {
  const encoded = encodeURIComponent(cloudProject);
  return request<CloudSaveResponse>(`/api/cloud/projects/${encoded}/save/${projectId}`, {
    method: 'PUT',
  });
}

/** List IFC/BIM model files in a project's models/ directory. */
export async function cloudListModels(project: string): Promise<CloudFile[]> {
  const encoded = encodeURIComponent(project);
  const res = await request<{ files: CloudFile[] }>(`/api/cloud/projects/${encoded}/models`);
  return res.files;
}

/** Read the project manifest (project.wefc). */
export async function cloudReadManifest(project: string): Promise<unknown> {
  const encoded = encodeURIComponent(project);
  return request<unknown>(`/api/cloud/projects/${encoded}/manifest`);
}
