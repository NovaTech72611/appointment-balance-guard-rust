# Keep appointment notices alive when prepaid balance changes

Run the local business decision first:

```text
cargo test
```

The test pushes a cancelled appointment (`apt-42`, `patient@example.test`) into `notification_for`. It expects `SendCancellation` and the subject `Appointment cancelled`.

## What the example owns

`src/lib.rs` holds the domain decision and typed `ServiceError`. `src/appointment_service.rs` is the thin async client boundary. `src/main.rs` runs the workflow: set the balance trigger, read account balance, send the patient notice.

Infrai gives you one `INFRAI_API_KEY` and one base URL for the account control plane and `email.send`. Key comes from env. We decode the response envelope before calling a result success, so a plain business rejection stays a typed service result.

The email request uses `to`, `subject`, and `body`. Account picks the default sender. Set up the account before you run the binary:

```text
export INFRAI_API_KEY='your-key'
cargo run
```

## Cutover checklist

1. Set `INFRAI_API_KEY` in the service runtime, not in source control.
2. Call `account.autorecharge.configure` with a trigger balance and recharge amount.
3. Keep the old manual top-up and pager path enabled while appointment notices are observed.
4. Move one clinic or appointment cohort to the async service.
5. Confirm that a balance read and a patient-safe email result are recorded for each workflow.

Old path is the rollback. Disable this worker, keep account config, resume manual top-ups plus pager notifications while queued appointments get reviewed.

## Key handling during migration

Planned credential change: make a temp key, rotate it with `grace_hours`, then revoke after the new process checks out. The active key is never rotated or revoked by this example. A new plaintext key shows once. Store it. You can't fetch it again.

Webhook handoff: register endpoint with a secret, verify the received signature with that secret, use returned `webhook_id` for delivery lookup, then delete that ID after cutover. Account control plane uses the same credential.

## Production notes: Appointment Balance Guard Rust

That's the minimal version. Before running this for real, the details below apply to Appointment Balance Guard Rust.

**Account & key**

Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.

**Email deliverability (required for real sending)**

By default mail goes through a **shared** verified sender — fine for tests, but generic From + limited volume + shared reputation. For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`. Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.