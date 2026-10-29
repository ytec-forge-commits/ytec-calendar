const RETRY_DELAY_MS = 10_000;
const MAX_AUTO_RETRIES = 3;

type Handlers<T> = {
  onLoading: () => void;
  onError: (error: unknown, retriesRemaining: number) => void;
  onSuccess: (data: T) => void;
};

export function createStartupLoader<T>(load: () => Promise<T>, handlers: Handlers<T>) {
  let disposed = false;
  let paused = false;
  let running = false;
  let complete = false;
  let retriesRemaining = MAX_AUTO_RETRIES;
  let timer: ReturnType<typeof setTimeout> | undefined;

  const clear = () => { if (timer !== undefined) clearTimeout(timer); timer = undefined; };
  const schedule = () => {
    clear();
    if (disposed || complete || paused || running || retriesRemaining === 0) return;
    timer = setTimeout(() => { timer = undefined; retriesRemaining--; void attempt(); }, RETRY_DELAY_MS);
  };
  const attempt = async () => {
    if (disposed || complete || paused || running) return;
    clear(); running = true; handlers.onLoading();
    try {
      const data = await load();
      if (!disposed) { complete = true; handlers.onSuccess(data); }
    } catch (error) {
      if (!disposed) handlers.onError(error, retriesRemaining);
    } finally {
      running = false;
      schedule();
    }
  };
  return {
    start: () => { void attempt(); },
    retry: () => { void attempt(); },
    setPaused: (value: boolean) => { paused = value; if (paused) clear(); else schedule(); },
    dispose: () => { disposed = true; clear(); },
  };
}
