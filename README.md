# axiom-event-log-engine

## Purpose
The event log engine is the durable, ordered source of truth for all events in the Axiom platform.

It provides append-only persistence with defined ordering and durability guarantees and enables downstream consumers to replay events deterministically.

## Responsibilities
- Accept events from the ingest gateway
- Append events to an ordered log
- Ensure durability and consistency guarantees
- Manage partitions and log segments
- Expose read interfaces for downstream consumers

## Non-Responsibilities
- Business logic or event interpretation
- Stream processing or aggregation
- Client-facing APIs
- Query serving or indexing
- System control or orchestration

## Failure Model
- Correctness is prioritized over availability
- Data corruption is unacceptable
- On restart, must recover to a consistent state

## Status
Week 0: Minimal buildable binary proving Rust toolchain and runtime.
