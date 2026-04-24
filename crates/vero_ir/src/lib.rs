//! Vero intermediate representation.
//!
//! Vero source lowers into this IR. Runtime backends consume this IR.

use serde::{Deserialize, Serialize};

mod coherence;
pub use coherence::{check_genome, check_program, check_rule, check_weight, CoherenceError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeroProgram {
    pub version: u32,
    pub model: ModelSpec,
    pub phases: PhaseSpec,
    pub boundaries: BoundariesSpec,
    pub weights: Vec<WeightDecl>,
    pub genome: Option<GenomeDecl>,
    pub rules: Vec<RuleDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ModelSpec {
    #[serde(rename = "char_mvp")]
    CharMvp(CharMvpSpec),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharMvpSpec {
    pub hidden_size: usize,
    pub edges_per_readout: usize,
    pub ticks_per_char: usize,
    pub p_spike: f32,
    pub pulse_current: f32,
    pub plasticity_every: usize,
    pub lr: f32,
    pub graph_seed: u64,
    pub hidden_ring_weight_scale: f32,
    pub input_to_hidden_weight_mean: f32,
    pub input_to_hidden_weight_jitter: f32,
    /// Number of interior grid intervals per B-spline edge (G). Control points = G + degree.
    pub kan_n_grid: usize,
    /// Polynomial degree of KAN B-spline edges (1=linear, 2=quadratic, 3=cubic).
    pub kan_degree: usize,
    pub kan_init_scale: f32,
    /// CPPN hidden layer width. 0 disables CPPN init (falls back to random).
    pub cppn_hidden: usize,
    /// Seed for deterministic CPPN weight initialisation.
    pub cppn_seed: u64,
}

impl Default for CharMvpSpec {
    fn default() -> Self {
        Self {
            hidden_size: 256,
            edges_per_readout: 4,
            ticks_per_char: 6,
            p_spike: 0.35,
            pulse_current: 3.0,
            plasticity_every: 64,
            lr: 0.05,
            graph_seed: 0x5345434B414E5F30,
            hidden_ring_weight_scale: 0.2,
            input_to_hidden_weight_mean: 0.5,
            input_to_hidden_weight_jitter: 0.1,
            kan_n_grid: 5,
            kan_degree: 3,
            kan_init_scale: 0.3,
            cppn_hidden: 12,
            cppn_seed: 0x43505041_4E5F4D32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseSpec {
    pub inference_topology_locked: bool,
    pub plasticity_topology_mutable: bool,
}

impl Default for PhaseSpec {
    fn default() -> Self {
        Self {
            inference_topology_locked: true,
            plasticity_topology_mutable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundariesSpec {
    pub membrane_sum: DomainBoundarySpec,
    pub loss: DomainBoundarySpec,
}

impl Default for BoundariesSpec {
    fn default() -> Self {
        Self {
            membrane_sum: DomainBoundarySpec { lower: 0, upper: 5_000_000 },
            loss: DomainBoundarySpec { lower: 0, upper: 50_000 },
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DomainBoundarySpec {
    pub lower: i64,
    pub upper: i64,
}

// ─── Vero type annotations ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TruthAnnotation {
    Provable,
    Unprovable,
}

impl Default for TruthAnnotation {
    fn default() -> Self { Self::Provable }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HarmonyAnnotation {
    Coherent,
    Dissonant,
}

impl Default for HarmonyAnnotation {
    fn default() -> Self { Self::Coherent }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeAnnotation {
    Now,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceAnnotation {
    SubstrateCurrent,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WeightBounds {
    pub lower: f32,
    pub upper: f32,
}

/// A named, type-annotated B-spline weight declaration.
///
/// The four Vero base annotations (truth, harmony, time, place) are compile-time
/// constraints checked by the coherence checker before the IR is lowered to the runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeightDecl {
    pub name: String,
    pub value: f32,
    pub bounds: WeightBounds,
    pub truth: TruthAnnotation,
    pub harmony: HarmonyAnnotation,
    pub time: TimeAnnotation,
    pub place: PlaceAnnotation,
}

impl Default for WeightDecl {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            value: 0.0,
            bounds: WeightBounds { lower: -1.0, upper: 1.0 },
            truth: TruthAnnotation::Provable,
            harmony: HarmonyAnnotation::Coherent,
            time: TimeAnnotation::Now,
            place: PlaceAnnotation::SubstrateCurrent,
        }
    }
}

/// Genome declaration — specifies the evolutionary search configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenomeDecl {
    pub population_size: usize,
    pub speciation_threshold: f32,
    pub survival_rate: f32,
    pub crossover_rate: f32,
    pub weight_perturb_rate: f32,
    pub weight_perturb_sigma: f32,
    pub add_node_prob: f32,
    pub add_edge_prob: f32,
    pub stagnation_limit: usize,
    pub elite_per_species: usize,
    pub seed: u64,
    pub truth: TruthAnnotation,
    pub harmony: HarmonyAnnotation,
}

impl Default for GenomeDecl {
    fn default() -> Self {
        Self {
            population_size: 50,
            speciation_threshold: 3.0,
            survival_rate: 0.2,
            crossover_rate: 0.75,
            weight_perturb_rate: 0.8,
            weight_perturb_sigma: 0.1,
            add_node_prob: 0.03,
            add_edge_prob: 0.05,
            stagnation_limit: 15,
            elite_per_species: 1,
            seed: 0,
            truth: TruthAnnotation::Provable,
            harmony: HarmonyAnnotation::Coherent,
        }
    }
}

/// A symbolic plasticity rule — name + expression string + Vero type annotations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleDecl {
    pub name: String,
    pub expression: String,
    pub truth: TruthAnnotation,
    pub harmony: HarmonyAnnotation,
}

impl VeroProgram {
    pub fn new_default() -> Self {
        Self {
            version: 1,
            model: ModelSpec::CharMvp(CharMvpSpec::default()),
            phases: PhaseSpec::default(),
            boundaries: BoundariesSpec::default(),
            weights: Vec::new(),
            genome: None,
            rules: Vec::new(),
        }
    }
}
