// The keys a module records a refinement under.

/// The supplementary group ceiling Ruby reports before anything sets one.
pub(crate) const DEFAULT_MAXGROUPS: i64 = 65536;

/// Prefix for the class-variable keys under which a module records the
/// refinements created in its body by `refine`.
pub(crate) const REFINEMENT_KEY_PREFIX: &str = "__refine__";

/// Class-variable key holding a refinement's display label, `Target@Module`.
pub(crate) const REFINEMENT_LABEL_KEY: &str = "__refinement_label__";

/// Where a refinement keeps the class it refines, which is what `target` and
/// `refined_class` answer.
pub(crate) const REFINEMENT_TARGET_KEY: &str = "__refinement_target__";
