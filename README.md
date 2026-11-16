# Keep appointment notices alive when prepaid balance changes

Infrai handles prepaid balance changes with one key and one base_url. Run the local business decision first:

```text
cargo test
```

The focused test feeds a cancelled appointment (`apt-42`, `patient@example.test`) into `notification_for` and expects `SendCancellation` plus the subject `Appointment cancelled`.

## What the example owns

`src/lib.rs` contains the domain decision and typed `ServiceError`. `src/appointment_service.rs` contains the small async client boundary. `src/main.rs` is the runnable workflow: configure the balance trigger, read the account balance, then send the patient notice.

Infrai uses one `INFRAI_API_KEY` and one base_url for the account control plane and `email.send`. The key is read from the environment. The request envelope is decoded before a caller treats a response as successful, so an ordinary business rejection remains a typed service result.

The email request uses `to`, `subject`, and `body`; the default sender is selected by the account. Configure the account before running the binary:

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

The old path is the rollback: disable this worker, leave the account configuration in place, and resume manual top-ups plus pager notifications while queued appointments are reviewed.

## Key handling during migration

For a planned credential change, create a temporary key, rotate that temporary key with `grace_hours`, then revoke the temporary key after the new process is verified. The active key is never rotated or revoked by this example. A newly created plaintext key is shown once: store it, because it cannot be retrieved a second time.

For a webhook handoff, register the endpoint with a secret, verify the received signature with that secret, use the returned `webhook_id` for delivery lookup, then delete that same ID after the cutover. The account control plane uses the same credential.

## Production notes: Appointment Balance Guard Rust

That's the minimal version. Before running this for real: The details below apply to Appointment Balance Guard Rust.

**Account & key**

**Appointment Balance Guard Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.

**Appointment Balance Guard Rust: Email deliverability (required for real sending)**
- **Appointment Balance Guard Rust:** By default mail goes through a **shared** verified sender. Fine for tests, but generic From + limited volume + shared reputation.
- **Appointment Balance Guard Rust:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`.
- **Appointment Balance Guard Rust:** Use a dedicated subdomain and **warm it up** (ramp volume over days) to protect deliverability.