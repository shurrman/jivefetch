import { type FormEvent, useCallback, useEffect, useMemo, useState } from "react";

import { actOnTask, addTask, getEngineStatus, listTasks, removeTask } from "./api";
import { type Language, type TranslationKey, useI18n } from "./i18n";
import type { EngineStatus, QueueTask, TaskAction, TaskState } from "./types";

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
  outputMissing: "outputMissing",
  processSupervisorError: "processSupervisorError",
  schedulerError: "schedulerError",
  outputDirectoryError: "outputDirectoryError",
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

function errorKeyForReason(reason: unknown): TranslationKey {
  const code = typeof reason === "string" ? reason : reason instanceof Error ? reason.message : "";
  return backendErrorKeys[code] ?? "unexpectedError";
}

function stateClass(state: TaskState) {
  return `state state-${state}`;
}

function displayUrl(value: string) {
  try {
    const url = new URL(value);
    return { host: url.host, path: url.pathname || "/" };
  } catch {
    return { host: value, path: "" };
  }
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

function versionLabel(version: string | null, available: boolean, availableText: string, missingText: string) {
  return version ?? (available ? availableText : missingText);
}

export default function App() {
  const { language, setLanguage, t } = useI18n();
  const locale = localeByLanguage[language];
  const [tasks, setTasks] = useState<QueueTask[]>([]);
  const [engines, setEngines] = useState<EngineStatus | null>(null);
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<TranslationKey | null>(null);

  const reload = useCallback(async (surfaceError = true) => {
    try {
      const [nextTasks, nextEngines] = await Promise.all([listTasks(), getEngineStatus()]);
      setTasks(nextTasks);
      setEngines(nextEngines);
      if (surfaceError) setError(null);
    } catch (reason) {
      if (surfaceError) setError(errorKeyForReason(reason));
    }
  }, []);

  useEffect(() => {
    void reload();
    const timer = window.setInterval(() => void reload(false), 1000);
    return () => window.clearInterval(timer);
  }, [reload]);

  const taskCount = useMemo(() => tasks.length, [tasks]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!url.trim()) return;

    setBusy(true);
    setError(null);
    try {
      const created = await addTask(url.trim());
      setTasks((current) => [created, ...current]);
      setUrl("");
    } catch (reason) {
      setError(errorKeyForReason(reason));
    } finally {
      setBusy(false);
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

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <div className="brand-mark" aria-hidden="true">JF</div>
          <div>
            <strong>{t("appName")}</strong>
            <span>{t("tagline")}</span>
          </div>
        </div>

        <label className="language-control">
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
      </header>

      <section className="hero">
        <div>
          <div className="eyebrow"><span className="pulse" /> {t("foundation")}</div>
          <h1>{t("localFirst")}</h1>
          <p>{t("defaultLanguage")}</p>
        </div>

        <form className="url-form" onSubmit={submit}>
          <label htmlFor="media-url">{t("addUrl")}</label>
          <div className="url-row">
            <input
              id="media-url"
              type="url"
              required
              value={url}
              placeholder={t("urlPlaceholder")}
              onChange={(event) => setUrl(event.target.value)}
              autoComplete="url"
            />
            <button className="button button-primary" type="submit" disabled={busy}>
              {busy ? t("adding") : t("addToQueue")}
            </button>
          </div>
        </form>
      </section>

      {error ? (
        <div className="error-banner" role="alert">
          <span>{t(error)}</span>
          <button type="button" onClick={() => void reload()}>{t("refresh")}</button>
        </div>
      ) : null}

      <section className="status-grid" aria-label={t("foundation")}>
        <article>
          <span className="status-icon">DB</span>
          <div>
            <strong>{t("persistence")}</strong>
            <p>{t("persistenceText")}</p>
          </div>
        </article>
        <article>
          <span className={`status-icon ${engines?.ready ? "" : "status-icon-warning"}`}>▶</span>
          <div>
            <strong>{t("engines")}</strong>
            <p>
              {engines?.ready
                ? t("enginesReady").replace("{count}", String(engines.concurrency))
                : t("enginesMissing")}
            </p>
            <div className="engine-versions">
              <span>yt-dlp: {versionLabel(engines?.ytDlp.version ?? null, engines?.ytDlp.available ?? false, t("available"), t("missing"))}</span>
              <span>FFmpeg: {versionLabel(engines?.ffmpeg.version ?? null, engines?.ffmpeg.available ?? false, t("available"), t("missing"))}</span>
            </div>
          </div>
        </article>
      </section>

      {engines ? <div className="output-folder"><strong>{t("outputFolder")}:</strong> {engines.outputDirectory}</div> : null}

      <section className="queue-section">
        <div className="section-heading">
          <div><span>{t("queue")}</span><strong>{taskCount}</strong><small>{t("tasks")}</small></div>
          <button className="button button-ghost" type="button" onClick={() => void reload()}>{t("refresh")}</button>
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
              const taskError = task.errorCode ? backendErrorKeys[task.errorCode] : null;
              return (
                <article className="task-card" key={task.id}>
                  <div className="task-main">
                    <span className={stateClass(task.state)}>{t(task.state)}</span>
                    <div className="task-details">
                      <div className="task-url"><strong>{shown.host}</strong><span>{shown.path}</span></div>
                      {activeStates.has(task.state) || task.state === "completed" ? (
                        <div className="progress-row">
                          <div className="progress-track" aria-label={`${progress}% ${t("complete")}`}>
                            <span style={{ width: `${progress}%` }} />
                          </div>
                          <small>{progress}%</small>
                        </div>
                      ) : null}
                      <div className="transfer-meta">
                        <span>{t("downloaded")}: {formatBytes(task.downloadedBytes, locale)}{task.totalBytes ? ` / ${formatBytes(task.totalBytes, locale)}` : ""}</span>
                        {task.speed ? <span>{formatBytes(task.speed, locale)}/s</span> : null}
                        {task.eta !== null ? <span>{t("etaLabel")}: {task.eta}s</span> : null}
                      </div>
                      {task.outputPath ? <div className="task-output">{task.outputPath}</div> : null}
                      {taskError ? <div className="task-error">{t(taskError)}</div> : null}
                    </div>
                    <div className="task-meta">
                      <span>{t("created")} {new Date(task.createdAt * 1000).toLocaleString(locale)}</span>
                      <span>{t("attempt")} {task.attemptCount} · {t("revision")} {task.revision}</span>
                    </div>
                  </div>
                  <div className="task-actions">
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
    </main>
  );
}
