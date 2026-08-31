import {
  createContext,
  type PropsWithChildren,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";

export type Language = "en" | "ru" | "zh-CN";

const dictionaries = {
  en: {
    appName: "JiveFetch",
    tagline: "Your media. Your queue. Your rules.",
    foundation: "Foundation build",
    localFirst: "Local-first queue prototype",
    language: "Language",
    addUrl: "Add a media URL",
    urlPlaceholder: "https://example.com/media",
    addToQueue: "Add to queue",
    adding: "Adding…",
    queue: "Queue",
    tasks: "tasks",
    emptyTitle: "Your queue is ready",
    emptyText: "Add a URL above. Tasks are stored in SQLite and survive app restarts.",
    pause: "Pause",
    resume: "Resume",
    stop: "Stop",
    remove: "Remove",
    refresh: "Refresh",
    queued: "Queued",
    running: "Running",
    paused: "Paused",
    stopped: "Stopped",
    completed: "Completed",
    failed: "Failed",
    persistence: "SQLite persistence",
    persistenceText: "Queue state and revisions are owned by the Rust backend.",
    engineNext: "Downloader engine is the next milestone",
    engineNextText:
      "This build validates the desktop shell, localization, durable queue, and controls without pretending that a download has started.",
    defaultLanguage: "English is the default; your explicit choice is remembered.",
    created: "Created",
    revision: "Revision",
    invalidUrl: "Enter a valid absolute URL.",
    unsupportedScheme: "Only HTTP and HTTPS URLs are supported in this build.",
    missingHost: "The URL must include a host.",
    taskNotFound: "The task no longer exists. Refresh the queue.",
    revisionConflict: "The task changed. Refresh the queue and try again.",
    invalidAction: "That action is not available for the current task state.",
    stopBeforeRemove: "Stop the running task before removing it.",
    storageError: "The queue storage could not be updated.",
    clockError: "The system clock could not be read.",
    unexpectedError: "An unexpected error occurred. Refresh the queue and try again.",
  },
  ru: {
    appName: "JiveFetch",
    tagline: "Ваши медиа. Ваша очередь. Ваши правила.",
    foundation: "Базовая рабочая версия",
    localFirst: "Локальный прототип очереди",
    language: "Язык",
    addUrl: "Добавить ссылку на медиа",
    urlPlaceholder: "https://example.com/media",
    addToQueue: "Добавить в очередь",
    adding: "Добавление…",
    queue: "Очередь",
    tasks: "задач",
    emptyTitle: "Очередь готова",
    emptyText: "Добавьте ссылку выше. Задачи хранятся в SQLite и переживают перезапуск приложения.",
    pause: "Пауза",
    resume: "Продолжить",
    stop: "Остановить",
    remove: "Удалить",
    refresh: "Обновить",
    queued: "В очереди",
    running: "Выполняется",
    paused: "На паузе",
    stopped: "Остановлено",
    completed: "Завершено",
    failed: "Ошибка",
    persistence: "Хранение в SQLite",
    persistenceText: "Состоянием очереди и ревизиями владеет Rust-бэкенд.",
    engineNext: "Движок загрузки — следующий этап",
    engineNextText:
      "Эта версия проверяет desktop-оболочку, локализацию, устойчивую очередь и команды, не изображая несуществующую загрузку.",
    defaultLanguage: "По умолчанию используется английский; явный выбор запоминается.",
    created: "Создано",
    revision: "Ревизия",
    invalidUrl: "Введите корректный абсолютный URL.",
    unsupportedScheme: "В этой версии поддерживаются только HTTP- и HTTPS-ссылки.",
    missingHost: "В URL должно быть указано имя хоста.",
    taskNotFound: "Задача больше не существует. Обновите очередь.",
    revisionConflict: "Задача изменилась. Обновите очередь и повторите попытку.",
    invalidAction: "Это действие недоступно для текущего состояния задачи.",
    stopBeforeRemove: "Перед удалением остановите выполняющуюся задачу.",
    storageError: "Не удалось обновить хранилище очереди.",
    clockError: "Не удалось прочитать системное время.",
    unexpectedError: "Произошла непредвиденная ошибка. Обновите очередь и повторите попытку.",
  },
  "zh-CN": {
    appName: "JiveFetch",
    tagline: "你的媒体，你的队列，你的规则。",
    foundation: "基础可运行版本",
    localFirst: "本地优先队列原型",
    language: "语言",
    addUrl: "添加媒体链接",
    urlPlaceholder: "https://example.com/media",
    addToQueue: "加入队列",
    adding: "正在添加…",
    queue: "队列",
    tasks: "个任务",
    emptyTitle: "队列已就绪",
    emptyText: "请在上方添加链接。任务保存在 SQLite 中，应用重启后仍会保留。",
    pause: "暂停",
    resume: "继续",
    stop: "停止",
    remove: "移除",
    refresh: "刷新",
    queued: "排队中",
    running: "运行中",
    paused: "已暂停",
    stopped: "已停止",
    completed: "已完成",
    failed: "失败",
    persistence: "SQLite 持久化",
    persistenceText: "队列状态和修订号由 Rust 后端统一管理。",
    engineNext: "下载引擎是下一个里程碑",
    engineNextText:
      "此版本验证桌面外壳、本地化、持久队列和控制功能，不会伪装成已经开始下载。",
    defaultLanguage: "默认语言为英语；应用会记住你明确选择的语言。",
    created: "创建时间",
    revision: "修订号",
    invalidUrl: "请输入有效的绝对 URL。",
    unsupportedScheme: "此版本仅支持 HTTP 和 HTTPS 链接。",
    missingHost: "URL 必须包含主机名。",
    taskNotFound: "该任务已不存在，请刷新队列。",
    revisionConflict: "任务已发生变化，请刷新队列后重试。",
    invalidAction: "当前任务状态不支持此操作。",
    stopBeforeRemove: "移除前请先停止正在运行的任务。",
    storageError: "无法更新队列存储。",
    clockError: "无法读取系统时间。",
    unexpectedError: "发生意外错误，请刷新队列后重试。",
  },
} as const;

export type TranslationKey = keyof (typeof dictionaries)["en"];

interface I18nContextValue {
  language: Language;
  setLanguage: (language: Language) => void;
  t: (key: TranslationKey) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);
const storageKey = "jivefetch.language";

function storedLanguage(): Language {
  const value = window.localStorage.getItem(storageKey);
  return value === "ru" || value === "zh-CN" || value === "en" ? value : "en";
}

export function I18nProvider({ children }: PropsWithChildren) {
  const [language, setLanguageState] = useState<Language>(storedLanguage);

  const setLanguage = (next: Language) => {
    window.localStorage.setItem(storageKey, next);
    setLanguageState(next);
  };

  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  const value = useMemo<I18nContextValue>(
    () => ({
      language,
      setLanguage,
      t: (key) => dictionaries[language][key],
    }),
    [language],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const value = useContext(I18nContext);
  if (!value) {
    throw new Error("useI18n must be used inside I18nProvider");
  }
  return value;
}
