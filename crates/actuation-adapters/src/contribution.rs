//! Public intake for harness capability descriptors — the route the
//! contract's own intake promise implies. It has two faces sharing one law:
//! validation, where an outside author checks a descriptor document against
//! the capability schema and the catalog laws before submitting; and
//! contribution, where a validated descriptor that fills a declared
//! capability gap is received with a receipt an integrator can land into
//! `catalog/targets.json`. Neither face mutates the catalog: the bundled
//! catalog is compiled into this binary, and landing a contribution remains
//! an owner edit under the correction discipline. A contribution fills a
//! declared gap; correcting an already-declared capability is the owner's
//! own edit, not an intake.
use crate::admission::HarnessCapability;
use crate::catalog::NativeCatalog;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const CAPABILITY_VALIDATION_VERSION: &str = "actuation.harness-capability-validation/v1";
pub const CAPABILITY_CONTRIBUTION_VERSION: &str = "actuation.capability-contribution/v1";

/// Length of the digest carried in a contribution_ref (full sha256 travels in
/// the receipt's source block).
const CONTRIBUTION_REF_DIGEST_LEN: usize = 12;

fn check(name: &str, passed: bool, detail: String) -> Value {
    json!({
        "check": name,
        "result": if passed { "pass" } else { "fail" },
        "detail": detail,
    })
}

fn declared_slugs(catalog: &NativeCatalog) -> String {
    catalog
        .descriptors()
        .iter()
        .map(|d| d.slug())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The intake law: schema admission against `actuation.harness-capability/v1`
/// (closed event vocabulary, reversible seams, wake honesty, provenance),
/// slug alignment with the detection catalog, and coverage closure — a
/// contribution must fill a declared capability gap, never shadow a declared
/// capability and never name an undeclared harness. The answer is a named
/// validation document; `valid` is the conjunction of its checks.
pub fn validate_capability_document(catalog: &NativeCatalog, value: &Value) -> Value {
    let slug = value["harness_slug"].as_str().unwrap_or_default();
    let admitted = HarnessCapability::try_from(value.clone());
    let schema_check = check(
        "schema-admission",
        admitted.is_ok(),
        match &admitted {
            Ok(_) => "admitted against actuation.harness-capability/v1".to_owned(),
            Err(error) => format!("refused by the capability schema: {error}"),
        },
    );
    let known = !slug.is_empty() && catalog.descriptor(slug).is_some();
    let slug_check = check(
        "slug-alignment",
        known,
        if known {
            format!(
                "harness_slug {slug} names a detection descriptor in catalog r{}",
                catalog.revision()
            )
        } else if slug.is_empty() {
            "the document declares no harness_slug".to_owned()
        } else {
            format!(
                "harness_slug {slug} names no detection descriptor in catalog r{}; declared: {}",
                catalog.revision(),
                declared_slugs(catalog),
            )
        },
    );
    let shadowing = known && catalog.capability(slug).is_some();
    let fills_gap = known && !shadowing && catalog.capability_gap(slug).is_some();
    let closure_check = check(
        "coverage-closure",
        fills_gap,
        if fills_gap {
            format!("landing {slug} fills its declared capability gap, preserving coverage closure")
        } else if shadowing {
            format!(
                "{slug} already carries a declared capability; a contribution must fill a \
                 declared capability gap — correcting a declared capability is an owner edit to \
                 catalog/targets.json under the correction discipline"
            )
        } else {
            format!(
                "{slug} declares no capability gap to fill; every detection descriptor must \
                 carry a capability or a declared gap, and intake cannot author the missing \
                 counterpart"
            )
        },
    );
    let checks = vec![schema_check, slug_check, closure_check];
    let valid = checks.iter().all(|c| c["result"] == json!("pass"));
    json!({
        "schema": CAPABILITY_VALIDATION_VERSION,
        "document": "capability-validation",
        "valid": valid,
        "harness_slug": slug,
        "catalog_revision": catalog.revision(),
        "checks": checks,
    })
}

/// The intake of a validated contribution: the receipt that holds a
/// capability descriptor until the owner lands it. The receipt carries the
/// contributed bytes' sha256, the descriptor and its provenance unchanged,
/// the validation checks, and the landing edit the owner applies to
/// `catalog/targets.json` (capabilities += descriptor, capability_gaps -=
/// slug, catalog_revision -> next). Refuses anything the validation law
/// refuses; the caller surfaces the validation document as the refusal body.
pub fn capability_contribution_receipt(
    catalog: &NativeCatalog,
    value: &Value,
    origin: &str,
    contributed_bytes: &str,
    received_at_unix_ms: i64,
) -> Result<Value, crate::Error> {
    let validation = validate_capability_document(catalog, value);
    if validation["valid"] != json!(true) {
        return Err(crate::Error::new(
            "contribution refused by the intake law (see the validation document on stdout)",
        ));
    }
    let slug = validation["harness_slug"].as_str().unwrap_or_default();
    let mut digest = Sha256::new();
    digest.update(contributed_bytes.as_bytes());
    let sha256 = format!("{:x}", digest.finalize());
    let current_revision = catalog.revision();
    let landed_revision = current_revision + 1;
    let capability = HarnessCapability::try_from(value.clone()).expect("validation admitted it");
    Ok(json!({
        "schema": CAPABILITY_CONTRIBUTION_VERSION,
        "document": "capability-contribution",
        "contribution_ref": format!("capability-contribution:{slug}:{}", &sha256[..CONTRIBUTION_REF_DIGEST_LEN]),
        "status": "received",
        "received_at_unix_ms": received_at_unix_ms,
        "harness_slug": slug,
        "fills_declared_capability_gap": true,
        "source": {"origin": origin, "sha256": sha256},
        "capability": capability.as_value(),
        "validation": validation,
        "landing": {
            "catalog_file": "catalog/targets.json",
            "current_revision": current_revision,
            "landed_revision": landed_revision,
            "edit": format!(
                "capabilities += the {slug} capability; capability_gaps -= {slug}; \
                 catalog_revision {current_revision} -> {landed_revision}"
            ),
            "merged_by": "owner — the bundled catalog is compiled into the binary, so landing \
                          a contribution is an owner edit to catalog/targets.json shipped in \
                          the next catalog revision"
        },
        "obligations": [
            "Actuation performs no catalog mutation: this receipt holds the contribution; the \
             owner reviews its declared facts against its cited sources and lands it into \
             catalog/targets.json.",
            "The contributor's provenance travels with the descriptor unchanged; the owner's \
             landing is the act that gives the facts standing in the shipped catalog.",
        ],
    }))
}
