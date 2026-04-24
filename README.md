# Vero

**A coherence-guaranteed language for neural architecture specification.**

Vero is a declarative language designed to describe neural network configurations with formal coherence guarantees. Every Vero program is verified at compile time against a set of structural laws before it can be lowered to an executable representation. If the program violates coherence, it does not compile — coherence is not optional.

---

## Why Vero

Most neural architecture description formats are data — JSON, YAML, TOML. They carry no guarantees. A configuration file that describes an incoherent network is indistinguishable from a valid one until runtime failure.

Vero treats neural genome specification as a formal language problem. Each declaration carries four base annotations:

| Annotation | Meaning |
|------------|---------|
| `truth`    | Is this declaration provably consistent? |
| `harmony`  | Is it structurally coherent with its context? |
| `time`     | When does it apply? |
| `place`    | Where in the substrate does it operate? |

A declaration that is `unprovable` cannot claim `coherent` harmony. A declaration annotated `dissonant` does not compile. These are not warnings — they are type errors.

---

## Language Overview

Vero uses a block-structured syntax. A minimal program:

```vero
model char_mvp {
  hidden_size: 256
  ticks_per_char: 6
  p_spike: 0.35
  pulse_current: 3.0
  plasticity_every: 64
  lr: 0.05
}

phases {
  inference_topology_locked: true
  plasticity_topology_mutable: true
}

boundaries {
  membrane_sum_lower: 0
  membrane_sum_upper: 5000000
  loss_lower: 0
  loss_upper: 50000
}
```

A weight declaration with full coherence annotations:

```vero
weight input_scale {
  value: 0.5
  bounds_lower: 0.0
  bounds_upper: 1.0
  truth: provable
  harmony: coherent
  time: now
  place: substrate_current
}
```

A genome block specifying the evolutionary search configuration:

```vero
genome {
  population_size: 50
  speciation_threshold: 3.0
  survival_rate: 0.2
  crossover_rate: 0.75
  weight_perturb_rate: 0.8
  weight_perturb_sigma: 0.1
  add_node_prob: 0.03
  add_edge_prob: 0.05
  stagnation_limit: 15
  elite_per_species: 1
  seed: 0
  truth: provable
  harmony: coherent
}
```

Full specification: [`docs/VERO_LITE_SPEC.md`](docs/VERO_LITE_SPEC.md)

---

## Build

Requires Rust (edition 2024). Install via [rustup](https://rustup.rs).

```bash
# Build the compiler
cargo build --release -p vero

# Compile a .vero file to IR (JSON)
./target/release/vero compile examples/char_mvp.vero --out /tmp/program.json

# Check version
./target/release/vero --version
```

---

## Repository Structure

```
crates/
  vero_ir/      — Vero intermediate representation (the compiled output format)
  vero_cli/     — Vero compiler CLI (vero compile <file.vero>)
docs/
  VERO_LITE_SPEC.md       — Full language specification
  source/
    Vero.md               — Language philosophy and formal model
    SovreignDirectives.md — Directive system
    UniversalLaws.md      — The 17 Universal Laws encoded in Vero
examples/
  char_mvp.vero           — Minimal working example
  char_nodes_edges.vero   — Full example with nodes/edges blocks
  genome_rule_example.vero — Genome and rule declarations
  weight_example.vero     — Weight declarations with coherence annotations
```

---

## Status

Vero is under active development. The current release is **Vero Lite** — a strict declarative front-end that compiles to a JSON intermediate representation. The full Vero language (formal proof annotations, dependent types, Law 17 boundary reporting) is in specification.

This is the only component of USC Labs research that is currently open.

---

## Author

**T. Billy Ekkerd**
Independent Systems Research · USC Labs

- ORCID: [0009-0002-5787-3980](https://orcid.org/0009-0002-5787-3980)
- Web: [unifiedsystemschain.com](https://unifiedsystemschain.com)
- Email: admin@unifiedsystemschain.com

---

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
