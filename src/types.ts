export type TaskState =
  | "queued"
  | "running"
  | "paused"
  | "stopped"
  | "completed"
  | "failed";

export type TaskAction = "pause" | "resume" | "stop";

export interface QueueTask {
  id: string;
  url: string;
  state: TaskState;
  revision: number;
  createdAt: number;
  updatedAt: number;
}
