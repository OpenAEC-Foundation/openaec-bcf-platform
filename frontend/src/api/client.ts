// Typed API client for the BCF Platform backend

import type {
  Project, CreateProject,
  Topic, CreateTopic,
  Comment, CreateComment,
  Viewpoint,
  ApiKey, ApiKeyCreated,
  ImportSummary, User,
} from '../types/api';

const BASE = '';

function getHeaders(): HeadersInit {
  const headers: HeadersInit = { 'Content-Type': 'application/json' };
  const token = localStorage.getItem('bcf_token');
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  return headers;
}

async function request<T>(url: string, opts?: RequestInit): Promise<T> {
  const res = await fetch(BASE + url, {
    headers: getHeaders(),
    ...opts,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new ApiError(res.status, body.error || res.statusText);
  }
  if (res.status === 204) return undefined as unknown as T;
  return res.json();
}

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

// --- Auth ---
export const auth = {
  me: () => request<User>('/auth/me'),
  loginUrl: () => '/auth/login',
};

// --- Projects ---
export const projects = {
  list: () => request<Project[]>('/bcf/2.1/projects'),
  get: (id: string) => request<Project>(`/bcf/2.1/projects/${id}`),
  create: (data: CreateProject) =>
    request<Project>('/bcf/2.1/projects', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  update: (id: string, data: Partial<CreateProject>) =>
    request<Project>(`/bcf/2.1/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  delete: (id: string) =>
    request<void>(`/api/v1/projects/${id}`, { method: 'DELETE' }),
};

// --- Topics ---
export const topics = {
  list: (projectId: string) =>
    request<Topic[]>(`/bcf/2.1/projects/${projectId}/topics`),
  get: (projectId: string, topicId: string) =>
    request<Topic>(`/bcf/2.1/projects/${projectId}/topics/${topicId}`),
  create: (projectId: string, data: CreateTopic) =>
    request<Topic>(`/bcf/2.1/projects/${projectId}/topics`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  update: (projectId: string, topicId: string, data: Partial<CreateTopic>) =>
    request<Topic>(`/bcf/2.1/projects/${projectId}/topics/${topicId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  delete: (projectId: string, topicId: string) =>
    request<void>(`/bcf/2.1/projects/${projectId}/topics/${topicId}`, {
      method: 'DELETE',
    }),
};

// --- Comments ---
export const comments = {
  list: (projectId: string, topicId: string) =>
    request<Comment[]>(
      `/bcf/2.1/projects/${projectId}/topics/${topicId}/comments`
    ),
  create: (projectId: string, topicId: string, data: CreateComment) =>
    request<Comment>(
      `/bcf/2.1/projects/${projectId}/topics/${topicId}/comments`,
      { method: 'POST', body: JSON.stringify(data) }
    ),
  update: (projectId: string, topicId: string, commentId: string, data: { comment: string }) =>
    request<Comment>(
      `/bcf/2.1/projects/${projectId}/topics/${topicId}/comments/${commentId}`,
      { method: 'PUT', body: JSON.stringify(data) }
    ),
  delete: (projectId: string, topicId: string, commentId: string) =>
    request<void>(
      `/bcf/2.1/projects/${projectId}/topics/${topicId}/comments/${commentId}`,
      { method: 'DELETE' }
    ),
};

// --- Viewpoints ---
export const viewpoints = {
  list: (projectId: string, topicId: string) =>
    request<Viewpoint[]>(
      `/bcf/2.1/projects/${projectId}/topics/${topicId}/viewpoints`
    ),
  snapshotUrl: (projectId: string, topicId: string, viewpointId: string) =>
    `${BASE}/bcf/2.1/projects/${projectId}/topics/${topicId}/viewpoints/${viewpointId}/snapshot`,
};

// --- BCF Import/Export ---
export const bcf = {
  importZip: async (projectId: string, file: File): Promise<ImportSummary> => {
    const form = new FormData();
    form.append('file', file);
    const res = await fetch(`${BASE}/api/v1/projects/${projectId}/import-bcf`, {
      method: 'POST',
      headers: {
        ...(localStorage.getItem('bcf_token')
          ? { Authorization: `Bearer ${localStorage.getItem('bcf_token')}` }
          : {}),
      },
      body: form,
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new ApiError(res.status, body.error || res.statusText);
    }
    return res.json();
  },
  exportUrl: (projectId: string) => `${BASE}/api/v1/projects/${projectId}/export-bcf`,
};

// --- API Keys ---
export const apiKeys = {
  list: (projectId: string) =>
    request<ApiKey[]>(`/api/v1/projects/${projectId}/api-keys`),
  create: (projectId: string, data: { name: string; expires_at?: string }) =>
    request<ApiKeyCreated>(`/api/v1/projects/${projectId}/api-keys`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  delete: (projectId: string, keyId: string) =>
    request<void>(`/api/v1/projects/${projectId}/api-keys/${keyId}`, {
      method: 'DELETE',
    }),
};
