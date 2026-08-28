import { describe, expect, it } from "vitest";

import {
  assertDemoEnvelope,
  recoveryPermission,
  type DemoAttempt,
  type DemoEnvelope
} from "./demo";

function attempt(overrides: Partial<DemoAttempt> = {}): DemoAttempt {
  return {
    id: "sample:maya",
    clientName: "Maya Patel",
    scheduledFor: "2026-08-30T17:00:00Z",
    state: "unfinished",
    reason: "Left before the sample deposit step",
    consent: {
      email: true,
      wording: "Email me once about this booking if I leave before confirming.",
      recordedAt: "2026-08-28T17:00:00Z"
    },
    outcome: null,
    receipts: [],
    ...overrides
  };
}

describe("demo recovery policy", () => {
  it("allows one recovery only when email consent is recorded", () => {
    expect(recoveryPermission(attempt()).allowed).toBe(true);
    expect(
      recoveryPermission(
        attempt({ consent: { email: false, wording: null, recordedAt: null } })
      )
    ).toEqual({
      allowed: false,
      label: "Email not allowed",
      explanation: "No email consent was recorded. This recovery stays stopped."
    });
  });

  it("presents the delivered receipt as the end of the sample transition", () => {
    const recovered = attempt({
      state: "recovered",
      outcome: "Sample follow-up delivered",
      receipts: [
        {
          channel: "email",
          status: "delivered",
          detail: "Sample email accepted by the in-process demo mailbox.",
          occurredAt: "2026-08-28T18:00:00Z",
          simulated: true
        }
      ]
    });
    expect(recoveryPermission(recovered).label).toBe("Sample delivered");
    expect(recovered.receipts).toHaveLength(1);
  });
});

describe("demo seed contract", () => {
  it("accepts the three-attempt M1 sample", () => {
    const envelope: DemoEnvelope = {
      workspaceToken: "a".repeat(43),
      workspace: {
        id: "workspace",
        expiresAt: "2026-08-29T18:00:00Z",
        practice: { name: "North Star Coaching", timezone: "Europe/London" },
        service: {
          name: "45-minute focus session",
          durationMinutes: 45,
          depositCents: 3500,
          currency: "GBP"
        },
        attempts: [
          attempt(),
          attempt({
            id: "sample:jordan",
            clientName: "Jordan Lee",
            consent: { email: false, wording: null, recordedAt: null }
          }),
          attempt({ id: "sample:alex", clientName: "Alex Morgan", state: "completed" })
        ]
      }
    };
    expect(assertDemoEnvelope(envelope)).toBe(envelope);
  });

  it("rejects a partial sample response", () => {
    expect(() => assertDemoEnvelope({ workspaceToken: "token" })).toThrow(
      "sample workspace response is incomplete"
    );
  });
});
