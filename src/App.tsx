import { type FormEvent, useCallback, useEffect, useMemo, useState } from "react";

import { actOnTask, addTask, listTasks, removeTask } from "./api";
import { type Language, type TranslationKey, useI18n } from "./i18n";
import type { QueueTask, TaskAction, TaskState } from "./types";

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
};

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
    return { host: url.host, path: `${url.pathname}${url.search}` || "/" };
  } catch {
    return { host: value, path: "" };
  }
}

export default function App() {
  const { language, setLanguage, t } = useI18n();
  const [tasks, setTasks] = useState<QueueTask[]>([]);
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<TranslationKey | null>(null);

  const reload = useCallback(async () => {
    try {
      setTasks(await listTasks());
      setError(null);
    } catch (reason) {
      setError(errorKeyForReason(reason));
    }
  }, []);

  useEffect(() => {
    void reload();
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
          <div className="brand-mark" aria-hidden="true">
            JF
          </div>
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
          <div className="eyebrow">
            <span className="pulse" /> {t("foundation")}
          </div>
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
          <button type="button" onClick={() => void reload()}>
            {t("refresh")}
          </button>
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
          <span className="status-icon status-icon-muted">→</span>
          <div>
            <strong>{t("engineNext")}</strong>
            <p>{t("engineNextText")}</p>
          </div>
        </article>
      </section>

      <section className="queue-section">
        <div className="section-heading">
          <div>
            <span>{t("queue")}</span>
            <strong>{taskCount}</strong>
            <small>{t("tasks")}</small>
          </div>
          <button className="button button-ghost" type="button" onClick={() => void reload()}>
            {t("refresh")}
          </button>
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
              return (
                <article className="task-card" key={task.id}>
                  <div className="task-main">
                    <span className={stateClass(task.state)}>{t(task.state)}</span>
                    <div className="task-url">
                      <strong>{shown.host}</strong>
                      <span>{shown.path}</span>
                    </div>
                    <div className="task-meta">
                      <span>
                        {t("created")} {new Date(task.createdAt * 1000).toLocaleString(localeByLanguage[language])}
                      </span>
                      <span>
                        {t("revision")} {task.revision}
                      </span>
                    </div>
                  </div>
                  <div className="task-actions">
                    {task.state === "queued" || task.state === "running" ? (
                      <button type="button" onClick={() => void runAction(task, "pause")}>
                        {t("pause")}
                      </button>
                    ) : null}
                    {task.state === "paused" || task.state === "stopped" ? (
                      <button type="button" onClick={() => void runAction(task, "resume")}>
                        {t("resume")}
                      </button>
                    ) : null}
                    {task.state !== "stopped" && task.state !== "completed" ? (
                      <button type="button" onClick={() => void runAction(task, "stop")}>
                        {t("stop")}
                      </button>
                    ) : null}
                    <button className="danger" type="button" onClick={() => void remove(task)}>
                      {t("remove")}
                    </button>
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
