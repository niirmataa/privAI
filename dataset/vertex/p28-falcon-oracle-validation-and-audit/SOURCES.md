# p28 — Falcon Oracle Validation and Audit

## Purpose
Teach how to validate a Rust Falcon component against a trusted oracle, how to interpret that success correctly, and how to continue auditing engineering quality after the first green tests.

## Core topics
- role of trusted C oracles in Rust crypto validation
- why one passing oracle test is a milestone, not the end
- using FFI only in tests, not as runtime dependence
- follow-up validation: inverse, roundtrip, broader coverage
- crate-structure discipline and warning cleanup as part of crypto engineering trust

## Primary sources
- Falcon reference material and reference implementation behavior
- component-level transform descriptions used for oracle comparison planning

## Implementation sources
- pure Rust Falcon arithmetic experiments with no_std crate structure
- Rust test harnesses that compare component outputs to a trusted reference
- crate-configuration examples involving root attributes and test-only FFI use

## Security sources
- engineering notes on reference-divergence risk
- examples where early equivalence checks passed but later refactors weakened trust
- guidance on treating warnings and configuration mistakes as audit signals

## Dedicated training environment
- specially written Rust Falcon component stubs
- test-only FFI oracle harnesses
- synthetic crate-structure mistakes such as misplaced no_std attributes
- staged validation exercises: forward oracle, inverse oracle, roundtrip, and cleanup reruns

## Boundary note
This pack does not claim full Falcon security from a few passing tests. It teaches oracle-driven validation, audit discipline, and the difference between first success and completed engineering confidence.