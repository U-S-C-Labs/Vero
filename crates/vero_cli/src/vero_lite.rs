use vero_ir::{
    CharMvpSpec, GenomeDecl, HarmonyAnnotation, ModelSpec, PlaceAnnotation, RuleDecl,
    VeroProgram, TimeAnnotation, TruthAnnotation, WeightBounds, WeightDecl,
};

#[derive(Debug)]
pub struct VeroLiteError {
    pub msg: String,
    pub line: usize,
}

fn strip_comment(line: &str) -> &str {
    let mut s = line;
    if let Some(i) = s.find("//") {
        s = &s[..i];
    }
    if let Some(i) = s.find('#') {
        s = &s[..i];
    }
    s.trim()
}

fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    let (k, v) = line.split_once(':')?;
    Some((k.trim(), v.trim().trim_end_matches(',')))
}

pub fn parse_vero_lite(source: &str) -> Result<VeroProgram, VeroLiteError> {
    let mut prog = VeroProgram::new_default();

    #[derive(Clone, Debug, PartialEq)]
    enum Block {
        None,
        Model,
        Nodes,
        Edges,
        Phases,
        Boundaries,
        Weight(WeightDecl),
        Genome(GenomeDecl),
        Rule(RuleDecl),
    }
    let mut block = Block::None;

    for (idx0, raw) in source.lines().enumerate() {
        let line_no = idx0 + 1;
        let line = strip_comment(raw);
        if line.is_empty() {
            continue;
        }

        if line.ends_with('{') {
            let head = line.trim_end_matches('{').trim();
            block = match head {
                "model char_mvp" => Block::Model,
                "nodes" => Block::Nodes,
                "edges" => Block::Edges,
                "phases" => Block::Phases,
                "boundaries" => Block::Boundaries,
                s if s.starts_with("weight ") => {
                    let name = s[7..].trim();
                    Block::Weight(WeightDecl {
                        name: name.to_string(),
                        value: 0.0,
                        bounds: WeightBounds { lower: 0.0, upper: 0.0 },
                        truth: TruthAnnotation::Provable,
                        harmony: HarmonyAnnotation::Coherent,
                        time: TimeAnnotation::Now,
                        place: PlaceAnnotation::SubstrateCurrent,
                    })
                }
                "genome" => Block::Genome(GenomeDecl::default()),
                s if s.starts_with("rule ") => {
                    let name = s[5..].trim();
                    Block::Rule(RuleDecl {
                        name: name.to_string(),
                        expression: String::new(),
                        truth: TruthAnnotation::Provable,
                        harmony: HarmonyAnnotation::Coherent,
                    })
                }
                _ => {
                    return Err(VeroLiteError {
                        msg: format!("unknown block start: {head}"),
                        line: line_no,
                    });
                }
            };
            continue;
        }

        if line == "}" {
            match block {
                Block::Weight(ref decl) => {
                    if decl.value < decl.bounds.lower || decl.value > decl.bounds.upper {
                        return Err(VeroLiteError {
                            msg: format!(
                                "weight '{}': value {} is outside declared bounds [{}, {}]",
                                decl.name, decl.value, decl.bounds.lower, decl.bounds.upper
                            ),
                            line: line_no,
                        });
                    }
                    prog.weights.push(decl.clone());
                }
                Block::Genome(ref decl) => {
                    prog.genome = Some(decl.clone());
                }
                Block::Rule(ref decl) => {
                    if decl.harmony == HarmonyAnnotation::Dissonant {
                        return Err(VeroLiteError {
                            msg: format!("rule '{}': dissonant harmony does not compile", decl.name),
                            line: line_no,
                        });
                    }
                    prog.rules.push(decl.clone());
                }
                _ => {}
            }
            block = Block::None;
            continue;
        }

        let Some((k, v)) = parse_key_value(line) else {
            return Err(VeroLiteError {
                msg: "expected key: value".to_string(),
                line: line_no,
            });
        };

        match block {
            Block::Model => {
                let ModelSpec::CharMvp(ref mut spec) = prog.model;
                apply_char_mvp_kv(spec, k, v).map_err(|msg| VeroLiteError { msg, line: line_no })?;
            }
            Block::Nodes => {
                let ModelSpec::CharMvp(ref mut spec) = prog.model;
                match k {
                    "hidden_size" => {
                        spec.hidden_size = parse_usize(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected usize".to_string(), line: line_no })?;
                    }
                    _ => return Err(VeroLiteError { msg: format!("unknown nodes key: {k}"), line: line_no }),
                }
            }
            Block::Edges => {
                let ModelSpec::CharMvp(ref mut spec) = prog.model;
                match k {
                    "edges_per_readout" => {
                        spec.edges_per_readout = parse_usize(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected usize".to_string(), line: line_no })?;
                    }
                    _ => return Err(VeroLiteError { msg: format!("unknown edges key: {k}"), line: line_no }),
                }
            }
            Block::Phases => {
                match k {
                    "inference_topology_locked" => {
                        prog.phases.inference_topology_locked = parse_bool(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected bool".to_string(), line: line_no })?;
                    }
                    "plasticity_topology_mutable" => {
                        prog.phases.plasticity_topology_mutable = parse_bool(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected bool".to_string(), line: line_no })?;
                    }
                    _ => return Err(VeroLiteError { msg: format!("unknown phases key: {k}"), line: line_no }),
                }
            }
            Block::Boundaries => {
                match k {
                    "membrane_sum_lower" => {
                        prog.boundaries.membrane_sum.lower = parse_i64(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected i64".to_string(), line: line_no })?;
                    }
                    "membrane_sum_upper" => {
                        prog.boundaries.membrane_sum.upper = parse_i64(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected i64".to_string(), line: line_no })?;
                    }
                    "loss_lower" => {
                        prog.boundaries.loss.lower = parse_i64(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected i64".to_string(), line: line_no })?;
                    }
                    "loss_upper" => {
                        prog.boundaries.loss.upper = parse_i64(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected i64".to_string(), line: line_no })?;
                    }
                    _ => return Err(VeroLiteError { msg: format!("unknown boundaries key: {k}"), line: line_no }),
                }
            }
            Block::Weight(ref mut decl) => {
                match k {
                    "value" => {
                        decl.value = parse_f32(v)
                            .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?;
                    }
                    "verified" => {
                        if !v.starts_with("bounds(") || !v.ends_with(')') {
                            return Err(VeroLiteError {
                                msg: "expected bounds(<f32>, <f32>)".to_string(),
                                line: line_no,
                            });
                        }
                        let inner = &v[7..v.len() - 1];
                        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                        if parts.len() != 2 {
                            return Err(VeroLiteError {
                                msg: "expected two values in bounds".to_string(),
                                line: line_no,
                            });
                        }
                        decl.bounds.lower = parse_f32(parts[0])
                            .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?;
                        decl.bounds.upper = parse_f32(parts[1])
                            .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?;
                    }
                    "truth" => {
                        decl.truth = match v {
                            "provable" => TruthAnnotation::Provable,
                            "unprovable" => TruthAnnotation::Unprovable,
                            _ => return Err(VeroLiteError {
                                msg: "expected provable or unprovable".to_string(),
                                line: line_no,
                            }),
                        };
                    }
                    "harmony" => {
                        decl.harmony = match v {
                            "coherent" => HarmonyAnnotation::Coherent,
                            "dissonant" => HarmonyAnnotation::Dissonant,
                            _ => return Err(VeroLiteError {
                                msg: "expected coherent or dissonant".to_string(),
                                line: line_no,
                            }),
                        };
                    }
                    "time" => {
                        if v != "now" {
                            return Err(VeroLiteError {
                                msg: "expected now".to_string(),
                                line: line_no,
                            });
                        }
                        decl.time = TimeAnnotation::Now;
                    }
                    "place" => {
                        if v != "substrate.current" {
                            return Err(VeroLiteError {
                                msg: "expected substrate.current".to_string(),
                                line: line_no,
                            });
                        }
                        decl.place = PlaceAnnotation::SubstrateCurrent;
                    }
                    "compassion" => {
                        if v != "harmony" {
                            return Err(VeroLiteError {
                                msg: "expected harmony".to_string(),
                                line: line_no,
                            });
                        }
                    }
                    _ => return Err(VeroLiteError { msg: format!("unknown weight key: {k}"), line: line_no }),
                }
            }
            Block::Genome(ref mut decl) => {
                match k {
                    "population_size" => decl.population_size = parse_usize(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected usize".to_string(), line: line_no })?,
                    "speciation_threshold" => decl.speciation_threshold = parse_f32(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?,
                    "survival_rate" => decl.survival_rate = parse_f32(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?,
                    "crossover_rate" => decl.crossover_rate = parse_f32(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?,
                    "weight_perturb_rate" => decl.weight_perturb_rate = parse_f32(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?,
                    "weight_perturb_sigma" => decl.weight_perturb_sigma = parse_f32(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?,
                    "add_node_prob" => decl.add_node_prob = parse_f32(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?,
                    "add_edge_prob" => decl.add_edge_prob = parse_f32(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected f32".to_string(), line: line_no })?,
                    "stagnation_limit" => decl.stagnation_limit = parse_usize(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected usize".to_string(), line: line_no })?,
                    "elite_per_species" => decl.elite_per_species = parse_usize(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected usize".to_string(), line: line_no })?,
                    "seed" => decl.seed = parse_u64(v)
                        .ok_or_else(|| VeroLiteError { msg: "expected u64".to_string(), line: line_no })?,
                    "truth" => decl.truth = match v {
                        "provable" => TruthAnnotation::Provable,
                        "unprovable" => TruthAnnotation::Unprovable,
                        _ => return Err(VeroLiteError { msg: "expected provable or unprovable".to_string(), line: line_no }),
                    },
                    "harmony" => decl.harmony = match v {
                        "coherent" => HarmonyAnnotation::Coherent,
                        "dissonant" => HarmonyAnnotation::Dissonant,
                        _ => return Err(VeroLiteError { msg: "expected coherent or dissonant".to_string(), line: line_no }),
                    },
                    _ => return Err(VeroLiteError { msg: format!("unknown genome key: {k}"), line: line_no }),
                }
            }
            Block::Rule(ref mut decl) => {
                match k {
                    "expression" => decl.expression = v.to_string(),
                    "truth" => decl.truth = match v {
                        "provable" => TruthAnnotation::Provable,
                        "unprovable" => TruthAnnotation::Unprovable,
                        _ => return Err(VeroLiteError { msg: "expected provable or unprovable".to_string(), line: line_no }),
                    },
                    "harmony" => decl.harmony = match v {
                        "coherent" => HarmonyAnnotation::Coherent,
                        "dissonant" => HarmonyAnnotation::Dissonant,
                        _ => return Err(VeroLiteError { msg: "expected coherent or dissonant".to_string(), line: line_no }),
                    },
                    _ => return Err(VeroLiteError { msg: format!("unknown rule key: {k}"), line: line_no }),
                }
            }
            Block::None => {
                return Err(VeroLiteError {
                    msg: "key/value outside of a block".to_string(),
                    line: line_no,
                });
            }
        }
    }

    Ok(prog)
}

fn apply_char_mvp_kv(spec: &mut CharMvpSpec, k: &str, v: &str) -> Result<(), String> {
    match k {
        "hidden_size" => spec.hidden_size = parse_usize(v).ok_or("expected usize")?,
        "edges_per_readout" => spec.edges_per_readout = parse_usize(v).ok_or("expected usize")?,
        "ticks_per_char" => spec.ticks_per_char = parse_usize(v).ok_or("expected usize")?,
        "p_spike" => spec.p_spike = parse_f32(v).ok_or("expected f32")?,
        "pulse_current" => spec.pulse_current = parse_f32(v).ok_or("expected f32")?,
        "plasticity_every" => spec.plasticity_every = parse_usize(v).ok_or("expected usize")?,
        "lr" => spec.lr = parse_f32(v).ok_or("expected f32")?,
        "graph_seed" => spec.graph_seed = parse_u64(v).ok_or("expected u64")?,
        "hidden_ring_weight_scale" => spec.hidden_ring_weight_scale = parse_f32(v).ok_or("expected f32")?,
        "input_to_hidden_weight_mean" => spec.input_to_hidden_weight_mean = parse_f32(v).ok_or("expected f32")?,
        "input_to_hidden_weight_jitter" => spec.input_to_hidden_weight_jitter = parse_f32(v).ok_or("expected f32")?,
        "kan_n_grid" => spec.kan_n_grid = parse_usize(v).ok_or("expected usize")?,
        "kan_degree" => spec.kan_degree = parse_usize(v).ok_or("expected usize")?,
        "kan_init_scale" => spec.kan_init_scale = parse_f32(v).ok_or("expected f32")?,
        "cppn_hidden" => spec.cppn_hidden = parse_usize(v).ok_or("expected usize")?,
        "cppn_seed" => spec.cppn_seed = parse_u64(v).ok_or("expected u64")?,
        _ => return Err(format!("unknown model key: {k}")),
    }
    Ok(())
}

fn parse_bool(s: &str) -> Option<bool> {
    match s {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_usize(s: &str) -> Option<usize> {
    s.parse::<usize>().ok()
}

fn parse_i64(s: &str) -> Option<i64> {
    s.parse::<i64>().ok()
}

fn parse_u64(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}

fn parse_f32(s: &str) -> Option<f32> {
    s.parse::<f32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENOME_RULE_EXAMPLE: &str = r#"
model char_mvp {
    hidden_size: 128
    cppn_hidden: 12
    cppn_seed: 0x435050414E5F4D32
}

genome {
    population_size: 30
    speciation_threshold: 3.0
    survival_rate: 0.2
    seed: 0xDEADBEEF
    truth: provable
    harmony: coherent
}

rule hebbian {
    expression: delta_w = lr * pre * post
    truth: provable
    harmony: coherent
}

rule rate_regulation {
    expression: delta_w = -0.01 * (rate - target_rate)
    truth: provable
    harmony: coherent
}

weight plasticity_lr {
    value: 0.05
    verified: bounds(0.0, 1.0)
    truth: provable
    harmony: coherent
    time: now
    place: substrate.current
    compassion: harmony
}
"#;

    #[test]
    fn genome_rule_roundtrip() {
        let prog = parse_vero_lite(GENOME_RULE_EXAMPLE)
            .expect("parse should succeed");

        // Genome parsed correctly
        let genome = prog.genome.as_ref().expect("genome block should exist");
        assert_eq!(genome.population_size, 30);
        assert!((genome.speciation_threshold - 3.0).abs() < 1e-6);
        assert_eq!(genome.seed, 0xDEADBEEF);

        // Rules parsed correctly
        assert_eq!(prog.rules.len(), 2);
        assert_eq!(prog.rules[0].name, "hebbian");
        assert!(prog.rules[0].expression.contains("pre * post"));
        assert_eq!(prog.rules[1].name, "rate_regulation");

        // Weights parsed correctly
        assert_eq!(prog.weights.len(), 1);
        assert_eq!(prog.weights[0].name, "plasticity_lr");
        assert!((prog.weights[0].value - 0.05).abs() < 1e-6);

        // Coherence passes
        assert!(vero_ir::check_program(&prog).is_ok(),
            "coherence check should pass");

        // JSON round-trip preserves genome
        let json = serde_json::to_string(&prog).expect("serialise");
        let prog2: vero_ir::VeroProgram = serde_json::from_str(&json).expect("deserialise");
        let genome2 = prog2.genome.as_ref().expect("genome survives JSON");
        assert_eq!(genome2.population_size, 30);
        assert_eq!(prog2.rules.len(), 2);
    }

    #[test]
    fn dissonant_rule_rejected() {
        // The parser enforces truth: provable|unprovable; dissonant is rejected at parse time.
        // Either a parse error or a coherence error is acceptable — the key is that
        // a rule with dissonant truth never silently succeeds end-to-end.
        let src = r#"
model char_mvp {
    hidden_size: 64
}
rule bad {
    expression: x
    truth: dissonant
    harmony: coherent
}
"#;
        match parse_vero_lite(src) {
            Err(_) => {} // rejected at parse — correct
            Ok(prog) => {
                assert!(vero_ir::check_program(&prog).is_err(),
                    "dissonant rule must fail coherence check if parse succeeds");
            }
        }
    }
}
