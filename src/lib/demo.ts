export const DEMO_STORAGE_KEY = "demo:workspace-token";

export type AttemptState = "unfinished" | "recovered" | "completed";

export interface DemoReceipt {
  readonly channel: "email" | "sms";
  readonly status: "accepted" | "delivered" | "bounced" | "failed";
  readonly detail: string;
  readonly occurredAt: string;
  readonly simulated: boolean;
}

export interface DemoAttempt {
  readonly id: string;
  readonly clientName: string;
  readonly scheduledFor: string;
  readonly state: AttemptState;
  readonly reason: string;
  readonly consent: {
    readonly email: boolean;
    readonly wording: string | null;
    readonly recordedAt: string | null;
  };
  readonly outcome: string | null;
  readonly receipts: readonly DemoReceipt[];
}

export interface DemoWorkspace {
  readonly id: string;
  readonly expiresAt: string;
  readonly practice: {
    readonly name: string;
    readonly timezone: string;
  };
  readonly service: {
    readonly name: string;
    readonly durationMinutes: number;
    readonly depositCents: number;
    readonly currency: string;
  };
  readonly attempts: readonly DemoAttempt[];
}

export interface DemoEnvelope {
  readonly workspaceToken: string;
  readonly workspace: DemoWorkspace;
}

export interface RecoveryPermission {
  readonly allowed: boolean;
  readonly label: string;
  readonly explanation: string;
}

export class DemoApiError extends Error {
  readonly code: string;
  readonly status: number;

  constructor(code: string, message: string, status: number) {
    super(message);
    this.name = "DemoApiError";
    this.code = code;
    this.status = status;
  }
}

export function recoveryPermission(attempt: DemoAttempt): RecoveryPermission {
  if (!attempt.consent.email) {
    return {
      allowed: false,
      label: "Email not allowed",
      explanation: "No email consent was recorded. This recovery stays stopped."
    };
  }
  if (attempt.state === "completed") {
    return {
      allowed: false,
      label: "Booking complete",
      explanation: "The booking is complete, so it does not need a follow-up."
    };
  }
  if (attempt.state === "recovered") {
    return {
      allowed: false,
      label: "Sample delivered",
      explanation: "The sample follow-up has a delivery receipt."
    };
  }
  return {
    allowed: true,
    label: "Ready for sample recovery",
    explanation: "Recorded email consent permits one sample follow-up."
  };
}

export function assertDemoEnvelope(value: unknown): DemoEnvelope {
  if (!isRecord(value) || typeof value.workspaceToken !== "string") {
    throw new Error("The sample workspace response is incomplete.");
  }
  const workspace = value.workspace;
  if (
    !isRecord(workspace) ||
    typeof workspace.id !== "string" ||
    typeof workspace.expiresAt !== "string" ||
    !isRecord(workspace.practice) ||
    typeof workspace.practice.name !== "string" ||
    !isRecord(workspace.service) ||
    typeof workspace.service.name !== "string" ||
    !Array.isArray(workspace.attempts) ||
    workspace.attempts.length !== 3
  ) {
    throw new Error("The sample workspace response is incomplete.");
  }
  return value as unknown as DemoEnvelope;
}

export async function createDemo(): Promise<DemoEnvelope> {
  return requestDemo("/api/v1/demo/workspaces", "POST");
}

export async function loadDemo(token: string): Promise<DemoEnvelope> {
  return requestDemo("/api/v1/demo/workspace", "GET", token);
}

export async function resetDemo(token: string): Promise<DemoEnvelope> {
  return requestDemo("/api/v1/demo/reset", "POST", token);
}

export async function recoverDemoAttempt(
  token: string,
  attemptId: string
): Promise<DemoEnvelope> {
  return requestDemo(
    `/api/v1/demo/attempts/${encodeURIComponent(attemptId)}/recover`,
    "POST",
    token
  );
}

async function requestDemo(
  path: string,
  method: "GET" | "POST",
  token?: string
): Promise<DemoEnvelope> {
  const headers = new Headers({ Accept: "application/json" });
  if (method === "POST") {
    headers.set("Idempotency-Key", crypto.randomUUID());
  }
  if (token) {
    headers.set("X-Demo-Workspace", token);
  }

  let response: Response;
  try {
    response = await fetch(path, { method, headers });
  } catch {
    throw new DemoApiError(
      "offline",
      "The sample workspace needs a connection. Check your network and try again.",
      0
    );
  }
  const value: unknown = await response.json().catch(() => null);
  if (!response.ok) {
    const code = isRecord(value) && typeof value.error === "string" ? value.error : "demo_unavailable";
    const message =
      isRecord(value) && typeof value.message === "string"
        ? value.message
        : "The sample workspace could not be loaded. Try again.";
    throw new DemoApiError(code, message, response.status);
  }
  return assertDemoEnvelope(value);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
