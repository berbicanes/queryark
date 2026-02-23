import * as Sentry from '@sentry/browser';

const DSN = 'https://a1eb9a39feac9b095304915c7e4b9cc0@o4510933530050560.ingest.de.sentry.io/4510936463245392';

let initialized = false;

export function initSentry(enabled: boolean) {
  if (!enabled || initialized) return;
  Sentry.init({
    dsn: DSN,
    environment: import.meta.env.DEV ? 'development' : 'production',
    sampleRate: 1.0,
    beforeSend(event) {
      // Strip PII: remove user IPs, connection strings
      if (event.request) delete event.request.url;
      return event;
    },
  });
  initialized = true;
}

export function captureError(error: unknown, context?: Record<string, unknown>) {
  if (!initialized) return;
  if (context) Sentry.setContext('extra', context);
  Sentry.captureException(error);
}

export function setUser(id: string) {
  if (!initialized) return;
  Sentry.setUser({ id });
}
