use crate::{GenomeDecl, HarmonyAnnotation, RuleDecl, VeroProgram, TruthAnnotation, WeightDecl};

#[derive(Debug, Clone)]
pub struct CoherenceError {
    pub weight_name: String,
    pub reason: String,
}

impl std::fmt::Display for CoherenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "coherence violation in weight '{}': {}", self.weight_name, self.reason)
    }
}

/// Check all declarations in a VeroProgram for coherence.
/// Returns Ok(()) if all pass, or Err with ALL violations (not just the first).
pub fn check_program(prog: &VeroProgram) -> Result<(), Vec<CoherenceError>> {
    let mut errors: Vec<CoherenceError> = prog.weights.iter()
        .flat_map(|w| check_weight(w))
        .collect();
    if let Some(ref g) = prog.genome {
        errors.extend(check_genome(g));
    }
    errors.extend(prog.rules.iter().flat_map(|r| check_rule(r)));
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Check a GenomeDecl for coherence violations.
pub fn check_genome(g: &GenomeDecl) -> Vec<CoherenceError> {
    let mut errs = Vec::new();
    if g.harmony == HarmonyAnnotation::Dissonant {
        errs.push(CoherenceError {
            weight_name: "genome".to_string(),
            reason: "dissonant harmony does not compile".to_string(),
        });
    }
    if g.population_size == 0 {
        errs.push(CoherenceError {
            weight_name: "genome".to_string(),
            reason: "population_size must be greater than zero".to_string(),
        });
    }
    if g.survival_rate <= 0.0 || g.survival_rate > 1.0 {
        errs.push(CoherenceError {
            weight_name: "genome".to_string(),
            reason: format!("survival_rate {} must be in (0.0, 1.0]", g.survival_rate),
        });
    }
    errs
}

/// Check a RuleDecl for coherence violations.
pub fn check_rule(r: &RuleDecl) -> Vec<CoherenceError> {
    let mut errs = Vec::new();
    if r.name.is_empty() || r.name == "unnamed" {
        errs.push(CoherenceError {
            weight_name: r.name.clone(),
            reason: "rule must have a non-empty, non-default name".to_string(),
        });
    }
    if r.harmony == HarmonyAnnotation::Dissonant {
        errs.push(CoherenceError {
            weight_name: r.name.clone(),
            reason: "dissonant harmony does not compile".to_string(),
        });
    }
    if r.expression.trim().is_empty() {
        errs.push(CoherenceError {
            weight_name: r.name.clone(),
            reason: "expression cannot be empty".to_string(),
        });
    }
    errs
}

/// Check a single WeightDecl. Returns a Vec of all violations found.
pub fn check_weight(w: &WeightDecl) -> Vec<CoherenceError> {
    let mut errs = Vec::new();

    // Rule 1: value must be within declared bounds
    if w.value < w.bounds.lower || w.value > w.bounds.upper {
        errs.push(CoherenceError {
            weight_name: w.name.clone(),
            reason: format!(
                "value {} is outside declared bounds [{}, {}]",
                w.value, w.bounds.lower, w.bounds.upper
            ),
        });
    }

    // Rule 2: Unprovable truth annotation is incoherent with Coherent harmony
    if w.truth == TruthAnnotation::Unprovable && w.harmony == HarmonyAnnotation::Coherent {
        errs.push(CoherenceError {
            weight_name: w.name.clone(),
            reason: "unprovable truth cannot claim coherent harmony".to_string(),
        });
    }

    // Rule 3: Dissonant harmony does not compile
    if w.harmony == HarmonyAnnotation::Dissonant {
        errs.push(CoherenceError {
            weight_name: w.name.clone(),
            reason: "dissonant harmony does not compile".to_string(),
        });
    }

    // Rule 4: name must not be empty or "unnamed"
    if w.name.is_empty() || w.name == "unnamed" {
        errs.push(CoherenceError {
            weight_name: w.name.clone(),
            reason: "weight must have a non-empty, non-default name".to_string(),
        });
    }

    errs
}
