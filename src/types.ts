export type TaskState =
  | "queued"
  | "starting"
  | "downloading"
  | "postprocessing"
  | "pausing"
  | "paused"
  | "stopping"
  | "stopped"
  | "completed"
  | "failed"
  | "interrupted";

export type TaskAction = "pause" | "resume" | "stop";

export interface QueueTask {
  id: string;
  url: string;
  state: TaskState;
  revision: number;
  createdAt: number;
  updatedAt: number;
  progress: number;
  downloadedBytes: number;
  totalBytes: number | null;
  speed: number | null;
  eta: number | null;
  outputPath: string | null;
  errorCode: string | null;
  attemptCount: number;
}

export interface EngineInfo {
  available: boolean;
  version: string | null;
}

export interface EngineStatus {
  ready: boolean;
  ytDlp: EngineInfo;
  ffmpeg: EngineInfo;
}

export interface AppSettings {
  concurrency: number;
  speedLimitBytesPerSecond: number | null;
  browserForCookies: string | null;
  outputDirectory: string;
}

export type Theme = "system" | "light" | "dark";
