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

export interface MediaFormat {
  selector: string;
  formatId: string;
  width: number | null;
  height: number | null;
  fps: number | null;
  videoCodec: string | null;
  extension: string | null;
  bitrateKbps: number | null;
  fileSize: number | null;
  hasAudio: boolean;
}

export interface MediaProbe {
  title: string;
  duration: number | null;
  formats: MediaFormat[];
}

export type Theme = "system" | "light" | "dark";
