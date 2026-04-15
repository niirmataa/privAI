# p27 — Falcon Paper-to-Rust Extraction

## Purpose
Teach the model to read Falcon-style reference material, extract the mathematically essential parts, and plan a faithful Rust translation without collapsing into pattern copying.

## Core topics
- extracting implementation-critical meaning from a Falcon paper
- separating proof-level description from reference-code obligations
- preserving indexing, scaling, and transformation semantics
- wrapper types over floating-point values as domain-specific arithmetic
- trusted-oracle thinking before optimization

## Primary sources
- Falcon specification and reference paper material
- explanatory texts on FFT-based lattice-signature arithmetic
- mathematical descriptions of transform structure and normalization obligations

## Implementation sources
- pure Rust no_std Falcon component experiments
- safe-loop and slice-based translations of pointer-heavy reference logic
- educational wrapper types for precision-sensitive arithmetic

## Security sources
- notes on numerical faithfulness, representational drift, and reference-divergence risks
- examples where cryptographic numerics fail despite compiling cleanly

## Dedicated training environment
- specially written Falcon arithmetic stubs in Rust
- controlled reference-to-Rust translation exercises
- synthetic ports that are cleaner stylistically but wrong mathematically
- oracle-comparison harnesses for component-level equivalence checking

## Boundary note
This pack does not teach a whole Falcon implementation end to end. It teaches how to read the paper and reference intent well enough to begin a faithful Rust implementation.