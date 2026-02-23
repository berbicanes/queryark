import { v4 as uuidv4 } from 'uuid';

// Custom lightweight telemetry — Plausible-style event API
// Events: app_launch, connection_created, query_executed, feature_used
// No PII: only event name + anonymous session ID (random UUID per install)
// Gated on settingsStore.telemetryEnabled

let sessionId = '';
let enabled = false;

// Telemetry endpoint — defaults to no-op until configured
const TELEMETRY_ENDPOINT = '';

export function initTelemetry(isEnabled: boolean, existingSessionId: string): string {
  enabled = isEnabled;
  if (existingSessionId) {
    sessionId = existingSessionId;
  } else {
    sessionId = uuidv4();
  }
  return sessionId;
}

export function setTelemetryEnabled(isEnabled: boolean) {
  enabled = isEnabled;
}

export function trackEvent(name: string, props?: Record<string, string | number>): void {
  if (!enabled || !TELEMETRY_ENDPOINT) return;

  const payload = {
    event: name,
    session_id: sessionId,
    timestamp: new Date().toISOString(),
    props: props ?? {},
  };

  // Fire-and-forget — never blocks UI
  fetch(TELEMETRY_ENDPOINT, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  }).catch(() => {
    // Silently fail on network errors
  });
}

export function getSessionId(): string {
  return sessionId;
}
