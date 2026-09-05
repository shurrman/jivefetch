import { open } from "@tauri-apps/plugin-dialog";
import {
  type FormEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import {
  actOnTask,
  addTask,
  getEngineStatus,
  getSettings,
  listTasks,
  openOutput,
  probeMedia,
  removeTask,
  updateSettings,
} from "./api";
import { type Language, type TranslationKey, useI18n } from "./i18n";
import type {
  AppSettings,
  EngineStatus,
  MediaFormat,
  MediaProbe,
  QueueTask,
  TaskAction,
  TaskState,
  Theme,
} from "./types";

const localeByLanguage: Record<Language, string> = {
  en: "en-US",
  ru: "ru-RU",
  "zh-CN": "zh-CN",
};

const backendErrorKeys: Record<string, TranslationKey> = {
  invalidUrl: "invalidUrl",
  unsupportedScheme: "unsupportedScheme",
  missingHost: "missingHost",
  taskNotFound: "taskNotFound",
  revisionConflict: "revisionConflict",
  invalidAction: "invalidAction",
  stopBeforeRemove: "stopBeforeRemove",
  storageError: "storageError",
  clockError: "clockError",
  ytDlpMissing: "ytDlpMissing",
  ffmpegMissing: "ffmpegMissing",
  engineSpawnFailed: "engineSpawnFailed",
  engineFailed: "engineFailed",
  browserCookiesUnavailable: "browserCookiesUnavailable",
  authenticationRequired: "authenticationRequired",
  mediaUnavailable: "mediaUnavailable",
  rateLimited: "rateLimited",
  httpForbidden: "httpForbidden",
  formatUnavailable: "formatUnavailable",
  networkError: "networkError",
  permissionDenied: "permissionDenied",
  outputMissing: "outputMissing",
  openOutputFailed: "openOutputFailed",
  processSupervisorError: "processSupervisorError",
  schedulerError: "schedulerError",
  outputDirectoryError: "outputDirectoryError",
  invalidOutputDirectory: "invalidOutputDirectory",
  invalidConcurrency: "invalidConcurrency",
  invalidSpeedLimit: "invalidSpeedLimit",
  invalidCookieBrowser: "invalidCookieBrowser",
  invalidFormatSelection: "invalidFormatSelection",
  probeFailed: "probeFailed",
  probeTimedOut: "probeTimedOut",
  probeOutputInvalid: "probeOutputInvalid",
  noFormats: "noFormats",
  databaseTooNew: "databaseTooNew",
  interruptedAfterRestart: "interruptedAfterRestart",
};

const activeStates = new Set<TaskState>([
  "starting",
  "downloading",
  "postprocessing",
  "pausing",
  "stopping",
]);
const removableStates = new Set<TaskState>([
  "paused",
  "stopped",
  "completed",
  "failed",
  "interrupted",
]);
const failedStates = new Set<TaskState>(["failed", "interrupted"]);
const downloadStageKeys: Record<string, TranslationKey> = {
  video: "downloadingVideo",
  audio: "downloadingAudio",
  media: "downloadingMedia",
  postprocessing: "mergingMedia",
};
const concurrencyPresets = Array.from({ length: 10 }, (_, index) => index + 1);
const speedPresets = [512 * 1024, 1024 * 1024, 2 * 1024 * 1024, 3 * 1024 * 1024];
const themeStorageKey = "jivefetch.theme";
const sensitiveQueryKey = /(auth|cookie|credential|key|pass|policy|secret|session|sig|token)/i;

function storedTheme(): Theme {
  const value = window.localStorage.getItem(themeStorageKey);
  return value === "light" || value === "dark" ? value : "system";
}

function errorKeyForReason(reason: unknown): TranslationKey {
  const code = typeof reason === "string" ? reason : reason instanceof Error ? reason.message : "";
  return backendErrorKeys[code] ?? "unexpectedError";
}

function stateClass(task: QueueTask) {
  const state = task.state === "completed" && !task.outputAvailable ? "failed" : task.state;
  return `state state-${state}`;
}

function progressClass(task: QueueTask) {
  if (task.state === "completed" && task.outputAvailable) return "progress-track progress-completed";
  if (task.state === "completed" || failedStates.has(task.state)) return "progress-track progress-failed";
  if (activeStates.has(task.state)) return "progress-track progress-active";
  return "progress-track progress-idle";
}

function displayUrl(value: string) {
  try {
    const url = new URL(value);
    const parameters = new URLSearchParams();
    for (const [key, parameterValue] of url.searchParams) {
      parameters.append(key, sensitiveQueryKey.test(key) ? "redacted" : parameterValue);
    }
    const query = parameters.size > 0 ? `?${parameters.toString()}` : "";
    return { host: url.host, path: `${url.pathname || "/"}${query}${url.hash}` };
  } catch {
    return { host: value, path: "" };
  }
}

function formatMediaOption(format: MediaFormat, locale: string) {
  const parts: string[] = [];
  if (format.height) parts.push(`${format.height}p`);
  else if (format.width) parts.push(`${format.width}px`);
  if (format.fps) parts.push(`${new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }).format(format.fps)} fps`);
  if (format.bitrateKbps) {
    parts.push(`${new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }).format(format.bitrateKbps / 1000)} Mbps`);
  }
  if (format.videoCodec) parts.push(format.videoCodec.split(".")[0].toUpperCase());
  if (format.extension) parts.push(format.extension.toUpperCase());
  if (format.fileSize) parts.push(`≈ ${formatBytes(format.fileSize, locale)}`);
  return parts.join(" · ");
}

function formatBytes(value: number | null, locale: string) {
  if (value === null) return "—";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let shown = value;
  let index = 0;
  while (shown >= 1000 && index < units.length - 1) {
    shown /= 1000;
    index += 1;
  }
  return `${new Intl.NumberFormat(locale, { maximumFractionDigits: index === 0 ? 0 : 1 }).format(shown)} ${units[index]}`;
}

function formatDuration(seconds: number) {
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  if (minutes < 60) return `${minutes}m ${remainder}s`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}

function versionLabel(version: string | null, available: boolean, availableText: string, missingText: string) {
  return version ?? (available ? availableText : missingText);
}

export default function App() {
  const { language, setLanguage, t } = useI18n();
  const locale = localeByLanguage[language];
  const [tasks, setTasks] = useState<QueueTask[]>([]);
  const [engines, setEngines] = useState<EngineStatus | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [theme, setThemeState] = useState<Theme>(storedTheme);
  const [customConcurrencyMode, setCustomConcurrencyMode] = useState(false);
  const [customSpeedMode, setCustomSpeedMode] = useState(false);
  const [customConcurrency, setCustomConcurrency] = useState("11");
  const [customSpeedKiB, setCustomSpeedKiB] = useState("4096");
  const [url, setUrl] = useState("");
  const [probe, setProbe] = useState<MediaProbe | null>(null);
  const [probedUrl, setProbedUrl] = useState("");
  const [selectedFormat, setSelectedFormat] = useState("");
  const [probing, setProbing] = useState(false);
  const [contextMenu, setContextMenu] = useState<{ taskId: string; x: number; y: number } | null>(null);
  const [busy, setBusy] = useState(false);
  const [settingsBusy, setSettingsBusy] = useState(false);
  const [error, setError] = useState<TranslationKey | null>(null);
  const pollInFlight = useRef(false);

  const refreshTasks = useCallback(async (surfaceError = false) => {
    if (pollInFlight.current) return;
    pollInFlight.current = true;
    try {
      setTasks(await listTasks());
      if (surfaceError) setError(null);
    } catch (reason) {
      if (surfaceError) setError(errorKeyForReason(reason));
    } finally {
      pollInFlight.current = false;
    }
  }, []);

  const reload = useCallback(async (surfaceError = true) => {
    try {
      const [nextTasks, nextEngines, nextSettings] = await Promise.all([
        listTasks(),
        getEngineStatus(),
        getSettings(),
      ]);
      setTasks(nextTasks);
      setEngines(nextEngines);
      setSettings(nextSettings);
      if (nextSettings.concurrency > 10) setCustomConcurrency(String(nextSettings.concurrency));
      if (
        nextSettings.speedLimitBytesPerSecond !== null &&
        !speedPresets.includes(nextSettings.speedLimitBytesPerSecond)
      ) {
        setCustomSpeedKiB(String(Math.round(nextSettings.speedLimitBytesPerSecond / 1024)));
      }
      if (surfaceError) setError(null);
    } catch (reason) {
      if (surfaceError) setError(errorKeyForReason(reason));
    }
  }, []);

  useEffect(() => {
    void reload();
    const timer = window.setInterval(() => void refreshTasks(false), 5000);
    return () => window.clearInterval(timer);
  }, [refreshTasks, reload]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem(themeStorageKey, theme);
  }, [theme]);

  useEffect(() => {
    const closeMenu = () => setContextMenu(null);
    const preventBrowserMenu = (event: MouseEvent) => event.preventDefault();
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") closeMenu();
    };
    document.addEventListener("pointerdown", closeMenu);
    document.addEventListener("contextmenu", preventBrowserMenu);
    document.addEventListener("keydown", closeOnEscape);
    window.addEventListener("blur", closeMenu);
    return () => {
      document.removeEventListener("pointerdown", closeMenu);
      document.removeEventListener("contextmenu", preventBrowserMenu);
      document.removeEventListener("keydown", closeOnEscape);
      window.removeEventListener("blur", closeMenu);
    };
  }, []);

  const taskCount = useMemo(() => tasks.length, [tasks]);

  const saveSettings = async (next: AppSettings) => {
    setSettingsBusy(true);
    setError(null);
    try {
      setSettings(await updateSettings(next));
    } catch (reason) {
      setError(errorKeyForReason(reason));
    } finally {
      setSettingsBusy(false);
    }
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!url.trim()) return;
    setBusy(true);
    setError(null);
    try {
      const normalizedUrl = url.trim();
      const formatSelector = probedUrl === normalizedUrl && selectedFormat ? selectedFormat : null;
      const created = await addTask(normalizedUrl, formatSelector);
      setTasks((current) => [created, ...current]);
      setUrl("");
      setProbe(null);
      setProbedUrl("");
      setSelectedFormat("");
    } catch (reason) {
      setError(errorKeyForReason(reason));
    } finally {
      setBusy(false);
    }
  };

  const inspectFormats = async () => {
    const normalizedUrl = url.trim();
    if (!normalizedUrl) return;
    setProbing(true);
    setError(null);
    try {
      setProbe(await probeMedia(normalizedUrl));
      setProbedUrl(normalizedUrl);
      setSelectedFormat("");
    } catch (reason) {
      setProbe(null);
      setProbedUrl("");
      setSelectedFormat("");
      setError(errorKeyForReason(reason));
    } finally {
      setProbing(false);
    }
  };

  const runAction = async (task: QueueTask, action: TaskAction) => {
    setError(null);
    try {
      const updated = await actOnTask(task, action);
      setTasks((current) => current.map((item) => (item.id === updated.id ? updated : item)));
    } catch (reason) {
      setError(errorKeyForReason(reason));
    }
  };

  const remove = async (task: QueueTask) => {
    setError(null);
    try {
      await removeTask(task);
      setTasks((current) => current.filter((item) => item.id !== task.id));
    } catch (reason) {
      setError(errorKeyForReason(reason));
    }
  };

  const copyTaskUrl = async (task: QueueTask) => {
    setContextMenu(null);
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(task.url);
      } else {
        const textArea = document.createElement("textarea");
        textArea.value = task.url;
        textArea.style.position = "fixed";
        textArea.style.opacity = "0";
        document.body.appendChild(textArea);
        textArea.select();
        const copied = document.execCommand("copy");
        textArea.remove();
        if (!copied) throw new Error("clipboardUnavailable");
      }
    } catch {
      setError("unexpectedError");
    }
  };

  const openTaskOutput = async (task: QueueTask) => {
    setError(null);
    try {
      await openOutput(task.id);
    } catch (reason) {
      setError(errorKeyForReason(reason));
      await refreshTasks(false);
    }
  };

  const chooseOutputDirectory = async () => {
    if (!settings) return;
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: settings.outputDirectory,
        title: t("chooseOutputFolder"),
      });
      if (typeof selected === "string") {
        await saveSettings({ ...settings, outputDirectory: selected });
      }
    } catch (reason) {
      setError(errorKeyForReason(reason));
    }
  };

  const concurrencyValue = !customConcurrencyMode && settings && settings.concurrency <= 10
    ? String(settings.concurrency)
    : "custom";
  const speedValue = customSpeedMode
    ? "custom"
    : settings?.speedLimitBytesPerSecond === null
    ? "none"
    : settings && speedPresets.includes(settings.speedLimitBytesPerSecond ?? -1)
      ? String(settings.speedLimitBytesPerSecond)
      : "custom";
  const contextTask = contextMenu
    ? tasks.find((task) => task.id === contextMenu.taskId) ?? null
    : null;
  const canStartContextTask = contextTask
    ? ["paused", "stopped", "failed", "interrupted"].includes(contextTask.state)
    : false;
  const sizedFormats = probe?.formats.filter((format) => format.fileSize !== null) ?? [];
  const canPauseContextTask = contextTask
    ? ["queued", "starting", "downloading", "postprocessing"].includes(contextTask.state)
    : false;
  const canStopContextTask = contextTask
    ? ["queued", "starting", "downloading", "postprocessing", "pausing", "paused"].includes(contextTask.state)
    : false;

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <img className="brand-mark" src="/jivefetch-icon.png" alt="" />
          <div>
            <strong>{t("appName")}</strong>
            <span className="brand-status">
              {t("engineStatusLead")}
              <i className={`engine-readiness-dot ${engines?.ready ? "ready" : "missing"}`} aria-hidden="true" />
              {engines?.ready ? t("enginesReady") : t("enginesNotReady")}
            </span>
          </div>
        </div>
        <div className="engine-summary" aria-label={t("versionSummary")}>
          <span>JiveFetch <strong>{engines?.appVersion ?? "—"}</strong></span>
          <span>yt-dlp <strong>{versionLabel(engines?.ytDlp.version ?? null, engines?.ytDlp.available ?? false, t("available"), t("missing"))}</strong></span>
          <span>FFmpeg <strong>{versionLabel(engines?.ffmpeg.version ?? null, engines?.ffmpeg.available ?? false, t("available"), t("missing"))}</strong></span>
        </div>
      </header>

      <section className="control-strip" aria-label={t("settings")}>
        <label className="compact-control">
          <span>{t("concurrentTasks")}</span>
          <select
            value={concurrencyValue}
            disabled={!settings || settingsBusy}
            onChange={(event) => {
              if (!settings) return;
              if (event.target.value === "custom") {
                setCustomConcurrencyMode(true);
                return;
              }
              setCustomConcurrencyMode(false);
              void saveSettings({ ...settings, concurrency: Number(event.target.value) });
            }}
          >
            {concurrencyPresets.map((value) => <option key={value} value={value}>{value}</option>)}
            <option value="custom">{t("custom")}</option>
          </select>
          {concurrencyValue === "custom" ? (
            <input
              type="number"
              min="1"
              max="64"
              value={customConcurrency}
              aria-label={t("customConcurrency")}
              disabled={!settings || settingsBusy}
              onChange={(event) => setCustomConcurrency(event.target.value)}
              onBlur={() => {
                const value = Number(customConcurrency);
                if (settings && Number.isInteger(value)) {
                  void saveSettings({ ...settings, concurrency: value });
                }
              }}
            />
          ) : null}
        </label>

        <label className="compact-control">
          <span>{t("speedLimit")}</span>
          <select
            value={speedValue}
            disabled={!settings || settingsBusy}
            onChange={(event) => {
              if (!settings) return;
              if (event.target.value === "custom") {
                setCustomSpeedMode(true);
                return;
              }
              setCustomSpeedMode(false);
              const value = event.target.value === "none" ? null : Number(event.target.value);
              void saveSettings({ ...settings, speedLimitBytesPerSecond: value });
            }}
          >
            <option value="none">{t("unlimited")}</option>
            <option value={speedPresets[0]}>512 KB/s</option>
            <option value={speedPresets[1]}>1 MB/s</option>
            <option value={speedPresets[2]}>2 MB/s</option>
            <option value={speedPresets[3]}>3 MB/s</option>
            <option value="custom">{t("custom")}</option>
          </select>
          {speedValue === "custom" ? (
            <div className="unit-input">
              <input
                type="number"
                min="1"
                value={customSpeedKiB}
                aria-label={t("customSpeed")}
                disabled={!settings || settingsBusy}
                onChange={(event) => setCustomSpeedKiB(event.target.value)}
                onBlur={() => {
                  const value = Number(customSpeedKiB);
                  if (settings && Number.isFinite(value)) {
                    void saveSettings({ ...settings, speedLimitBytesPerSecond: Math.round(value * 1024) });
                  }
                }}
              />
              <small>KB/s</small>
            </div>
          ) : null}
        </label>

        <label className="compact-control">
          <span>{t("browserCookies")}</span>
          <select
            value={settings?.browserForCookies ?? "none"}
            disabled={!settings || settingsBusy}
            onChange={(event) => {
              if (!settings) return;
              void saveSettings({
                ...settings,
                browserForCookies: event.target.value === "none" ? null : event.target.value,
              });
            }}
          >
            <option value="none">{t("noBrowserCookies")}</option>
            <option value="brave">Brave</option>
            <option value="chrome">Chrome</option>
            <option value="chromium">Chromium</option>
            <option value="edge">Edge</option>
            <option value="firefox">Firefox</option>
            <option value="opera">Opera</option>
            <option value="safari">Safari</option>
            <option value="vivaldi">Vivaldi</option>
            <option value="whale">Whale</option>
          </select>
        </label>

        <label className="compact-control">
          <span>{t("theme")}</span>
          <select value={theme} onChange={(event) => setThemeState(event.target.value as Theme)}>
            <option value="system">{t("themeSystem")}</option>
            <option value="dark">{t("themeDark")}</option>
            <option value="light">{t("themeLight")}</option>
          </select>
        </label>

        <label className="compact-control">
          <span>{t("language")}</span>
          <select
            value={language}
            onChange={(event) => setLanguage(event.target.value as Language)}
            aria-label={t("language")}
          >
            <option value="en">English</option>
            <option value="ru">Русский</option>
            <option value="zh-CN">简体中文</option>
          </select>
        </label>

        <div className="folder-control">
          <span>{t("outputFolder")}</span>
          <button type="button" disabled={!settings || settingsBusy} onClick={() => void chooseOutputDirectory()}>
            <span>{settings?.outputDirectory ?? "—"}</span>
            <strong>{t("choose")}</strong>
          </button>
        </div>
      </section>

      <section className="hero">
        <form className="url-form" onSubmit={submit}>
          <label htmlFor="media-url">{t("addUrl")}</label>
          <div className="url-row">
            <input
              id="media-url"
              type="url"
              required
              value={url}
              placeholder={t("urlPlaceholder")}
              onChange={(event) => {
                setUrl(event.target.value);
                setProbe(null);
                setProbedUrl("");
                setSelectedFormat("");
              }}
              autoComplete="url"
            />
            <div className="url-actions">
              <button className="button button-ghost" type="button" disabled={probing || busy || !url.trim()} onClick={() => void inspectFormats()}>
                {probing ? t("inspectingFormats") : t("inspectFormats")}
              </button>
              <button className="button button-primary" type="submit" disabled={busy || probing}>
                {busy ? t("adding") : t("addToQueue")}
              </button>
            </div>
          </div>
          {probe && probedUrl === url.trim() ? (
            <div className="format-picker">
              <div>
                <strong>{probe.title}</strong>
                {probe.duration ? <small>{formatDuration(Math.round(probe.duration))}</small> : null}
              </div>
              <label>
                <span>{t("formatChoice")}</span>
                <select value={selectedFormat} onChange={(event) => setSelectedFormat(event.target.value)}>
                  <option value="">{t("maximumQuality")}</option>
                  {sizedFormats.map((format) => (
                    <option key={format.selector} value={format.selector}>
                      {formatMediaOption(format, locale)}
                    </option>
                  ))}
                </select>
              </label>
            </div>
          ) : null}
        </form>
      </section>

      {error ? (
        <div className="error-banner" role="alert">
          <span>{t(error)}</span>
          <button type="button" onClick={() => void reload()}>{t("refresh")}</button>
        </div>
      ) : null}

      <section className="queue-section">
        <div className="section-heading">
          <div><span>{t("queue")}</span><strong>{taskCount}</strong><small>{t("tasks")}</small></div>
          <div className="queue-refresh">
            <small>{t("autoRefresh")}</small>
            <button className="button button-ghost" type="button" onClick={() => void refreshTasks(true)}>{t("refresh")}</button>
          </div>
        </div>

        {tasks.length === 0 ? (
          <div className="empty-state">
            <div aria-hidden="true">＋</div>
            <h2>{t("emptyTitle")}</h2>
            <p>{t("emptyText")}</p>
          </div>
        ) : (
          <div className="task-list">
            {tasks.map((task) => {
              const shown = displayUrl(task.url);
              const progress = Math.round(task.progress * 100);
              const taskError = task.state === "completed" && !task.outputAvailable
                ? "outputMissing"
                : task.errorCode ? backendErrorKeys[task.errorCode] : null;
              const taskStage = task.downloadStage ? downloadStageKeys[task.downloadStage] : null;
              return (
                <article
                  className="task-card"
                  key={task.id}
                  onContextMenu={(event) => {
                    event.preventDefault();
                    setContextMenu({
                      taskId: task.id,
                      x: Math.min(event.clientX, window.innerWidth - 220),
                      y: Math.min(event.clientY, window.innerHeight - 230),
                    });
                  }}
                >
                  <div className="task-main">
                    <span className={stateClass(task)}>{t(task.state)}</span>
                    <div className="task-details">
                      <div className="task-url"><strong>{shown.host}</strong><span>{shown.path}</span></div>
                      <div className="progress-row">
                        <div className={progressClass(task)} aria-label={`${progress}% ${t("complete")}`}>
                          <span style={{ width: `${progress}%` }} />
                        </div>
                        <small>{progress}%</small>
                      </div>
                      <div className="transfer-meta">
                        <span>{t("downloaded")}: {formatBytes(task.downloadedBytes, locale)} / {formatBytes(task.totalBytes, locale)}</span>
                        <span>{t("speed")}: {task.speed ? `${formatBytes(task.speed, locale)}/s` : "—"}</span>
                        <span>{t("etaLabel")}: {task.eta !== null ? formatDuration(task.eta) : "—"}</span>
                      </div>
                      {taskStage && ["downloading", "postprocessing"].includes(task.state) ? (
                        <div className="task-stage">{t(taskStage)}</div>
                      ) : null}
                      {task.outputPath ? <div className="task-output">{task.outputPath}</div> : null}
                      {taskError ? <div className="task-error">{t(taskError)}</div> : null}
                    </div>
                    <div className="task-meta">
                      <span>{t("created")} {new Date(task.createdAt * 1000).toLocaleString(locale)}</span>
                      <span>{t("attempt")} {task.attemptCount} · {t("revision")} {task.revision}</span>
                    </div>
                  </div>
                  <div className="task-actions">
                    {task.state === "completed" && task.outputAvailable ? (
                      <button type="button" onClick={() => void openTaskOutput(task)}>{t("openFile")}</button>
                    ) : null}
                    {["queued", "starting", "downloading", "postprocessing"].includes(task.state) ? (
                      <button type="button" onClick={() => void runAction(task, "pause")}>{t("pause")}</button>
                    ) : null}
                    {["paused", "stopped", "failed", "interrupted"].includes(task.state) ? (
                      <button type="button" onClick={() => void runAction(task, "resume")}>{t("resume")}</button>
                    ) : null}
                    {["queued", "starting", "downloading", "postprocessing", "pausing", "paused"].includes(task.state) ? (
                      <button type="button" onClick={() => void runAction(task, "stop")}>{t("stop")}</button>
                    ) : null}
                    {removableStates.has(task.state) ? (
                      <button className="danger" type="button" onClick={() => void remove(task)}>{t("remove")}</button>
                    ) : null}
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </section>

      {contextMenu && contextTask ? (
        <div
          className="task-context-menu"
          role="menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onPointerDown={(event) => event.stopPropagation()}
          onContextMenu={(event) => event.preventDefault()}
        >
          <button disabled={!canStartContextTask} onClick={() => { setContextMenu(null); void runAction(contextTask, "resume"); }}>{t("startTask")}</button>
          <button disabled={!canStopContextTask} onClick={() => { setContextMenu(null); void runAction(contextTask, "stop"); }}>{t("stop")}</button>
          <button disabled={!canPauseContextTask} onClick={() => { setContextMenu(null); void runAction(contextTask, "pause"); }}>{t("pause")}</button>
          <button onClick={() => void copyTaskUrl(contextTask)}>{t("copyUrl")}</button>
          <button className="danger" disabled={!removableStates.has(contextTask.state)} onClick={() => { setContextMenu(null); void remove(contextTask); }}>{t("remove")}</button>
        </div>
      ) : null}
    </main>
  );
}
