# p26 — FO Transform and CCA Thinking

## Purpose
Teach why CPA-level intuition is not enough for real network protocols, and how FO-style reasoning changes KEM integration, failure handling, and Rust implementation discipline.

## Core topics
- active attacker vs passive attacker mindset
- FO transform intuition
- CCA-aware KEM reasoning
- malformed ciphertext handling
- oracle-style leakage through errors, timing, or fallback behavior
- educational testing for CCA-aware Rust integrations

## Primary sources
- Fujisaki–Okamoto transform literature
- KEM security literature distinguishing CPA-style and CCA-style guarantees
- explanatory materials on active-attack security for public-key encapsulation

## Implementation sources
- Rust transport and handshake examples using KEM-style abstractions
- educational wrappers showing decapsulation, framing, and high-level protocol failure handling

## Security sources
- oracle-style attack case studies
- malformed-ciphertext handling notes
- examples where distinguishable failures weaken active-attack security

## Dedicated training environment
- specially written Rust handshake stubs with intentional bad failure branching
- synthetic malformed-ciphertext scenarios
- controlled tests for fallback behavior and distinguishable error handling
- small protocol exercises built only to teach CCA-aware reasoning

## Boundary note
This pack does not teach one exact production KEM API and does not memorize a single protocol spec. It teaches the attack model, the transform intuition, and the implementation consequences.