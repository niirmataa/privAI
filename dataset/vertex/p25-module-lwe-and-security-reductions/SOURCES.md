# p25 — Module-LWE and Security Reductions

## Purpose
Teach practical reasoning about LWE / Ring-LWE / Module-LWE, noise, reductions, and the implementation consequences of those ideas in Rust.

## Core topics
- plain LWE intuition
- Ring-LWE vs Module-LWE tradeoffs
- noise as a security-and-correctness parameter
- reduction-aware engineering intuition
- truncation and domain mistakes in Rust
- educational test design for toy lattice encryption

## Primary sources
- Regev, *On lattices, learning with errors, random linear codes, and cryptography*
- module-lattice / Module-LWE explanatory literature and scheme papers used in practical PQ systems
- educational lattice-cryptography texts on noise, modulus, and structured variants

## Implementation sources
- small Rust toy examples for modular arithmetic and explicit domain wrappers
- educational implementations showing reduced residues, centered values, and conversion boundaries

## Security sources
- literature and notes on parameter sensitivity, reduction scope, and implementation misuse
- examples of truncation, signedness, and domain-confusion bugs in arithmetic-heavy code

## Dedicated training environment
- specially written toy Rust LWE-style examples
- controlled examples with explicit truncation bugs
- negative tests showing noise too small / too large regimes
- synthetic review scenarios for reduction-aware reasoning

## Boundary note
This pack does not teach one specific production library API and does not memorize any single repository. It teaches the problem class and its implementation consequences.