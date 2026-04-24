# Vero-lite Spec (Draft)

Last updated: 2026-04-23

Goal: a strict declarative front-end for SECKAN that compiles to SECKAN IR.

Pipeline:
- Vero-lite source -> SECKAN IR (JSON) -> SECKAN runtime/backends (Rust)

---

## MVP Grammar (Current)

Top-level blocks:
- model char_mvp { ... }
- nodes { ... }
- edges { ... }
- phases { ... }
- boundaries { ... }

Comments:
- // ... and # ... are stripped.

Key/value lines:
- key: value

---

## model char_mvp keys

| Key                         | Type   | Default              | Description                                                  |
|-----------------------------|--------|----------------------|--------------------------------------------------------------|
| hidden_size                 | usize  | 256                  | Number of hidden (recurrent) neurons                         |
| edges_per_readout           | usize  | 4                    | Initial KAN edges connecting hidden → each readout node      |
| graph_seed                  | u64    | 0x5345434B414E5F30   | Seed for deterministic graph initialisation (hex supported)  |
| hidden_ring_weight_scale    | f32    | 0.2                  | Scale of random initial scalar ring weights                  |
| input_to_hidden_weight_mean | f32    | 0.5                  | Mean of random initial input→hidden scalar weights           |
| input_to_hidden_weight_jitter | f32  | 0.1                  | Jitter (±) on input→hidden scalar weights                   |
| kan_n_grid                  | usize  | 5                    | G: number of interior B-spline intervals per KAN edge        |
| kan_degree                  | usize  | 3                    | k: polynomial degree of B-spline KAN edges (cubic=3)         |
| kan_init_scale              | f32    | 0.3                  | Fallback random init scale (used only when cppn_hidden = 0)  |
| cppn_hidden                 | usize  | 12                   | CPPN hidden layer width; 0 = disable CPPN, use random init   |
| cppn_seed                   | u64    | 0x43505041_4E5F4D32  | Seed for deterministic CPPN weight initialisation            |
| ticks_per_char              | usize  | 6                    | LIF integration steps per input character                    |
| p_spike                     | f32    | 0.35                 | Bernoulli spike probability for input encoding               |
| pulse_current               | f32    | 3.0                  | Current magnitude for a Bernoulli spike pulse                |
| plasticity_every            | usize  | 64                   | Inference steps between plasticity updates                   |
| lr                          | f32    | 0.05                 | Learning rate for plasticity updates                         |

**Deprecated key (not accepted):** `kan_points` — replaced by `kan_n_grid` + `kan_degree`
as of M1 (B-spline upgrade, 2026-04-23).

Setting `cppn_hidden: 0` disables CPPN and reverts to the random init path (scaled by `kan_init_scale`).

---

## nodes keys (char_mvp)

| Key         | Type  | Default | Description              |
|-------------|-------|---------|--------------------------|
| hidden_size | usize | 256     | Number of hidden neurons |

---

## edges keys (char_mvp)

| Key               | Type  | Default | Description                               |
|-------------------|-------|---------|-------------------------------------------|
| edges_per_readout | usize | 4       | KAN edges per readout node at init        |

---

## phases keys

| Key                         | Type | Default | Description                                          |
|-----------------------------|------|---------|------------------------------------------------------|
| inference_topology_locked   | bool | true    | Topology mutations rejected during inference phase   |
| plasticity_topology_mutable | bool | true    | Topology mutations allowed during plasticity phase   |

Note: these values are parsed and stored in the IR. Runtime enforcement of the
phase permit system (PhaseScheduler) is M1 work.

---

## boundaries keys

| Key                  | Type | Default   | Description                                   |
|----------------------|------|-----------|-----------------------------------------------|
| membrane_sum_lower   | i64  | 0         | Lower bound for membrane-sum measurement      |
| membrane_sum_upper   | i64  | 5000000   | Upper bound for membrane-sum measurement      |
| loss_lower           | i64  | 0         | Lower bound for loss measurement              |
| loss_upper           | i64  | 50000     | Upper bound for loss measurement              |

Values outside these bounds are reported as Infinity/Unknown (Law 17 compliance).

---

## B-Spline Control Point Count

The number of control points per KAN edge is:
```
n_ctrl = kan_n_grid + kan_degree
```

Examples:
| kan_n_grid | kan_degree | n_ctrl | Notes                  |
|------------|------------|--------|------------------------|
| 5          | 3          | 8      | Default (cubic)        |
| 8          | 3          | 11     | Higher resolution      |
| 1          | 1          | 2      | Minimal (linear)       |
| 7          | 2          | 9      | Quadratic              |

---

## Worked Examples

### Minimal example (default model block only)
```vero
# Vero-lite example: char-level SECKAN MVP

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

### Full example (nodes/edges blocks, tuned B-spline)
```vero
# Vero-lite example: use nodes/edges blocks with B-spline tuning

model char_mvp {
  ticks_per_char: 6
  p_spike: 0.35
  pulse_current: 3.0
  plasticity_every: 64
  lr: 0.05
  graph_seed: 0x5345434B414E5F30
  hidden_ring_weight_scale: 0.25
  input_to_hidden_weight_mean: 0.55
  input_to_hidden_weight_jitter: 0.12
  kan_n_grid: 8
  kan_degree: 3
  kan_init_scale: 0.2
}

nodes {
  hidden_size: 256
}

edges {
  edges_per_readout: 8
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

---

## Compile and Run

```bash
# Compile a .vero file to JSON IR:
cd seckan && cargo run -q -p vero_cli -- compile examples/char_nodes_edges.vero \
  --out /tmp/char_mvp.seckan.json

# Run the compiled IR:
cd seckan && cargo run -q -p seckan_cli -- --program /tmp/char_mvp.seckan.json
```

Source files: `seckan/examples/char_mvp.vero`, `seckan/examples/char_nodes_edges.vero`

---

## weight block

Named, type-annotated B-spline weight declarations. Each `weight` block declares one
`WeightDecl` that is stored in the IR's `weights` array and validated by the coherence
checker before the JSON is written.

### Syntax

```vero
weight <name> {
    value: <f32>,
    verified: bounds(<lower_f32>, <upper_f32>),
    truth: provable | unprovable,
    harmony: coherent | dissonant,
    time: now,
    place: substrate.current,
    compassion: harmony,
}
```

### Fields

| Field       | Values                          | Description                                            |
|-------------|---------------------------------|--------------------------------------------------------|
| value       | f32                             | Declared initial value                                 |
| verified    | bounds(f32, f32)                | Lower and upper bounds (value must be within these)    |
| truth       | provable \| unprovable          | Whether the value can be proven from context           |
| harmony     | coherent \| dissonant           | Alignment state (dissonant does not compile — see coherence rules) |
| time        | now                             | Bound to current substrate state                       |
| place       | substrate.current               | Localised to active substrate                          |
| compassion  | harmony                         | Required ethical alignment field                       |

### Coherence Rules

All `WeightDecl`s are checked by `seckan_ir::check_program()` before the JSON IR is emitted:

1. **Bounds**: `value` must satisfy `lower ≤ value ≤ upper`.
2. **Truth-Harmony contract**: `truth: unprovable` combined with `harmony: coherent` is rejected — an unprovable claim cannot assert coherence.
3. **Dissonance rejection**: `harmony: dissonant` does not compile.
4. **Named**: `name` must not be empty or `"unnamed"`.

### Runtime Bindings

Weight names that map to `SeckanNetConfig` fields at runtime (applied after the `model` block):

| Weight name              | Config field                        | Type |
|--------------------------|-------------------------------------|------|
| `plasticity_lr`          | `cfg.plasticity_lr`                 | f32  |
| `kan_init_scale`         | `cfg.kan_init_scale`                | f32  |
| `hidden_ring_weight`     | `cfg.hidden_ring_weight_scale`      | f32  |
| `input_to_hidden_mean`   | `cfg.input_to_hidden_weight_mean`   | f32  |
| `input_to_hidden_jitter` | `cfg.input_to_hidden_weight_jitter` | f32  |
| `target_spike_rate`      | `cfg.target_spike_rate`             | f32  |
| `kan_y_clip`             | `cfg.kan_y_clip`                    | f32  |

Weight values **override** the corresponding `model char_mvp` keys. Any weight name not in
this table is accepted by the parser and coherence checker but produces a runtime warning
and has no effect on the network.

### Example

```vero
weight plasticity_lr {
    value: 0.07,
    verified: bounds(0.0, 1.0),
    truth: provable,
    harmony: coherent,
    time: now,
    place: substrate.current,
    compassion: harmony,
}
```

See `seckan/examples/weight_example.vero` for a full working example with five canonical weight declarations.

---

## Planned Extensions (M2+)

- `genome VeroMind { ... }` — encode the CPPN genome that initialises node coordinates
  and edge parameters from spatial position.
- `rule { ... }` — express plasticity update rules symbolically in Vero-lite
  rather than as hardcoded Rust.
