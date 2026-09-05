import { invoke } from "@tauri-apps/api/core";

import type { AppSettings, EngineStatus, MediaProbe, QueueTask, TaskAction } from "./types";

export function listTasks(): Promise<QueueTask[]> {
  return invoke<QueueTask[]>("list_tasks");
}

export function getEngineStatus(): Promise<EngineStatus> {
  return invoke<EngineStatus>("engine_status");
}

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("update_settings", { settings });
}

export function probeMedia(url: string): Promise<MediaProbe> {
  return invoke<MediaProbe>("probe_url", { url });
}

export function addTask(url: string, formatSelector: string | null): Promise<QueueTask> {
  return invoke<QueueTask>("add_task", { url, formatSelector });
}

export function actOnTask(task: QueueTask, action: TaskAction): Promise<QueueTask> {
  return invoke<QueueTask>("task_action", {
    taskId: task.id,
    action,
    expectedRevision: task.revision,
  });
}

export function removeTask(task: QueueTask): Promise<void> {
  return invoke<void>("remove_task", {
    taskId: task.id,
    expectedRevision: task.revision,
  });
}

export function openOutput(taskId: string): Promise<void> {
  return invoke<void>("open_output", { taskId });
}
