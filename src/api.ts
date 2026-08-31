import { invoke } from "@tauri-apps/api/core";

import type { QueueTask, TaskAction } from "./types";

export function listTasks(): Promise<QueueTask[]> {
  return invoke<QueueTask[]>("list_tasks");
}

export function addTask(url: string): Promise<QueueTask> {
  return invoke<QueueTask>("add_task", { url });
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
