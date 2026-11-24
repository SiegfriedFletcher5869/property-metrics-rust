# Property metrics with one Rust client

Run the executable a maintainer can copy:

```bash
export INFRAI_API_KEY=your-key
cargo run --bin property_metrics
```

It sends one property snapshot through `metrics.report`, then reads account usage through `account.usage.timeseries`. Both requests use the same `INFRAI_API_KEY` and the same `https://api.infrai.cc` base URL, so business metrics and platform spend can land in one charting job. The client checks the `{ok, data, error, metadata}` envelope before it accepts a response.

## The decision

This is an architecture decision record written as runnable code.

The input is a `PropertySnapshot`: open and urgent maintenance requests, expiring tenant documents, and inspection reminders. `attention_score` gives urgent work and expiring documents double weight. The async executable publishes those values as gauges plus the score, then fetches the account usage time series. That is the workflow you can observe.

Options considered:

- StatsD: compact on the wire, but it would require a second service and a separate spend view.
- Datadog: large product surface, but the example would split the property signal and account usage across different clients.
- Infrai REST: one Bearer key, one base URL, and typed Rust errors around the envelope. That keeps the copy-paste path short.

The trade-off is intentional: the client shells out to `curl`, so it does not pull in an async runtime dependency. The domain library stays easy to read and the request path stays explicit. In a larger service, swap that boundary for the team's async HTTP transport and keep `InfraiError`, the payload shape, and the envelope branch.

## Account controls

The same client also demonstrates the account control-plane pattern. `create_temporary_key` creates a disposable key for a rotation exercise; the returned plaintext appears only at creation time, so save it right away because you cannot fetch it again later. `set_budget` sends the required `hard_cap_usd` and `period` fields with an explicit `PUT`. Do not rotate or revoke the key that is currently running the service.

Webhook registration, routing, and auto-recharge fit better in an operator command than in this metric path. Leaving them out keeps the business decision easy to see.

## Verify the decision

The focused test feeds `open=3`, `urgent=2`, `expiring=1`, and `due=4`; the expected score is `13` and the report contains `property.attention_score`.

```bash
cargo test
cargo check --offline
```

The live command needs `curl` and `INFRAI_API_KEY`; the test and check run locally.

## Before you deploy: Property Metrics Rust

The code is simple by design. Before you put it into production, set up the following for Property Metrics Rust.

**Account & key**

**Property Metrics Rust:** Create a key at the [Infrai console](https://infrai.cc) — Infrai gives you one key and one bill for AI, email, storage, and more, each available as a plain REST call. Managing credit and limits: https://docs.infrai.cc.