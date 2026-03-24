// BCF Platform API types — mirrors the Rust backend models

export interface Project {
  project_id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateProject {
  name: string;
  description?: string;
}

export interface Topic {
  guid: string;
  project_id: string;
  title: string;
  description: string;
  topic_type: string;
  topic_status: string;
  priority: string;
  assigned_to: string | null;
  stage: string;
  labels: string[];
  due_date: string | null;
  index: number | null;
  creation_author: string | null;
  modified_author: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTopic {
  title: string;
  description?: string;
  topic_type?: string;
  topic_status?: string;
  priority?: string;
  assigned_to?: string;
  stage?: string;
  labels?: string[];
  due_date?: string;
}

export interface Comment {
  guid: string;
  topic_id: string;
  author_id: string | null;
  comment: string;
  viewpoint_guid: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateComment {
  comment: string;
  viewpoint_guid?: string;
}

export interface Viewpoint {
  guid: string;
  topic_id: string;
  snapshot_url: string | null;
  camera: Camera | null;
  components: unknown | null;
  created_at: string;
}

export interface Camera {
  camera_type: string;
  x: number;
  y: number;
  z: number;
  dir_x: number;
  dir_y: number;
  dir_z: number;
  up_x: number;
  up_y: number;
  up_z: number;
  fov: number | null;
  aspect: number | null;
}

export interface ApiKey {
  id: string;
  project_id: string;
  name: string;
  prefix: string;
  created_by: string | null;
  expires_at: string | null;
  created_at: string;
}

export interface ApiKeyCreated extends ApiKey {
  key: string;
}

export interface ImportSummary {
  topics_imported: number;
  comments_imported: number;
  viewpoints_imported: number;
}

export interface User {
  user_id: string;
  email: string;
  name: string;
  avatar_url?: string;
}
