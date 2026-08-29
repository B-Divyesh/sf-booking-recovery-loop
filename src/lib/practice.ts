export const PRACTICE_TOKEN_KEY = "practice:access-token";
export const RECEIPT_TOKEN_KEY = "practice:receipt-token";

export interface DeliveryEvent { channel: "email" | "sms"; status: "accepted" | "delivered" | "bounced" | "failed"; detail: string; occurredAt: string; }
export interface PracticeAttempt { id: string; clientName: string; email?: string; phone?: string; scheduledFor: string; state: string; emailConsent: boolean; smsConsent: boolean; consentWording: string; consentRecordedAt: string; events: DeliveryEvent[]; }
export interface Practice { id: string; name: string; publicSlug: string; timezone: string; serviceName: string; durationMinutes: number; depositCents: number; currency: string; paymentUrl: string; deliveryWebhookUrl: string; attempts: PracticeAttempt[]; }
export interface PublicPractice { name: string; publicSlug: string; timezone: string; serviceName: string; durationMinutes: number; depositCents: number; currency: string; paymentUrl: string; consentWording: string; }

async function responseJson<T>(response: Response): Promise<T> {
  const data = await response.json().catch(() => ({})) as T & { message?: string };
  if (!response.ok) throw new Error(data.message ?? "The request did not finish. Try again.");
  return data;
}

export async function createPractice(payload: Record<string, unknown>): Promise<{ accessToken: string; receiptToken: string; practice: Practice }> {
  return responseJson(await fetch("/api/v1/practices", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(payload) }));
}
export async function loadPractice(token: string): Promise<Practice> {
  return responseJson(await fetch("/api/v1/practice", { headers: { Authorization: `Bearer ${token}` } }));
}
export async function recoverPracticeAttempt(token: string, id: string): Promise<void> {
  await responseJson(await fetch(`/api/v1/practice/attempts/${encodeURIComponent(id)}/recover`, { method: "POST", headers: { Authorization: `Bearer ${token}` } }));
}
export async function publicPractice(slug: string): Promise<PublicPractice> { return responseJson(await fetch(`/api/v1/public/${encodeURIComponent(slug)}`)); }
export async function createBookingAttempt(slug: string, payload: Record<string, unknown>): Promise<{ attemptId: string; paymentUrl: string; status: string }> {
  return responseJson(await fetch(`/api/v1/public/${encodeURIComponent(slug)}/attempts`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(payload) }));
}
export async function deletePractice(token: string): Promise<void> {
  const response = await fetch("/api/v1/practice", { method: "DELETE", headers: { Authorization: `Bearer ${token}` } });
  if (!response.ok) throw new Error("Practice deletion did not finish. Try again.");
}
