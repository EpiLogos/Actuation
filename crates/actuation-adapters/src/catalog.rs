use crate::{admission::*, Error, Result};
use actuation_stream::{BoundaryCatalogue, NativeBoundary};
use serde_json::{json, Value};
use std::{collections::HashSet, path::Path};

/// A versioned collection of native declarations. Its revision describes the
/// declared facts, not the language used to load them. Loading never probes or
/// installs targets and does not imply their current availability.
#[derive(Clone, Debug)]
pub struct NativeCatalog {
    revision: i64,
    descriptors: Vec<HarnessDescriptor>,
    capabilities: Vec<HarnessCapability>,
    secrets: SecretSourceCatalog,
}
impl NativeCatalog {
    pub fn bundled() -> Result<Self> {
        Self::from_json(include_str!("../../../catalog/targets.json"))
    }
    pub fn from_path(path: &Path) -> Result<Self> {
        Self::from_json(
            &std::fs::read_to_string(path)
                .map_err(|e| Error::new(format!("catalog read failed: {e}")))?,
        )
    }
    pub fn from_json(source: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(source)?;
        require(
            v["schema"] == "actuation.native-catalog/v1",
            "wrong native catalog schema",
        )?;
        let catalog = HarnessCatalog::try_from(
            json!({"schema":HARNESS_DETECTION_VERSION,"document":"catalog","catalog_revision":v["catalog_revision"],"descriptors":v["descriptors"]}),
        )?;
        let revision = catalog.as_value()["catalog_revision"]
            .as_i64()
            .ok_or_else(|| Error::new("native catalog revision out of range"))?;
        require(revision > 0, "native catalog revision must be positive")?;
        let descriptors: Vec<HarnessDescriptor> = serde_json::from_value(v["descriptors"].clone())?;
        let capabilities: Vec<HarnessCapability> =
            serde_json::from_value(v["capabilities"].clone())?;
        let mut seen = HashSet::new();
        for c in &capabilities {
            let slug = text(&c.as_value()["harness_slug"])?;
            require(seen.insert(slug), "duplicate capability slug")?;
            require(
                descriptors.iter().any(|d| d.slug() == slug),
                "capability names an undeclared target",
            )?;
        }
        for d in &descriptors {
            require(
                d.as_value()["provenance"]["catalog_revision"]
                    .as_i64()
                    .is_some_and(|r| r <= revision),
                "descriptor provenance is newer than its catalog",
            )?;
        }
        Ok(Self {
            revision,
            descriptors,
            capabilities,
            secrets: SecretSourceCatalog::try_from(v["secret_sources"].clone())?,
        })
    }
    pub fn revision(&self) -> i64 {
        self.revision
    }
    pub fn descriptors(&self) -> &[HarnessDescriptor] {
        &self.descriptors
    }
    pub fn capabilities(&self) -> &[HarnessCapability] {
        &self.capabilities
    }
    pub fn descriptor(&self, slug: &str) -> Option<&HarnessDescriptor> {
        self.descriptors.iter().find(|d| d.slug() == slug)
    }
    pub fn capability(&self, slug: &str) -> Option<&HarnessCapability> {
        self.capabilities
            .iter()
            .find(|c| c.as_value()["harness_slug"] == slug)
    }
    pub fn secret_sources(&self) -> &SecretSourceCatalog {
        &self.secrets
    }
    pub fn read(&self) -> HarnessCatalog {
        HarnessCatalog::try_from(json!({"schema":HARNESS_DETECTION_VERSION,"document":"catalog","catalog_revision":self.revision,"descriptors":self.descriptors})).expect("admitted catalog")
    }
    pub fn select(&self, slugs: &[String]) -> Result<Vec<HarnessDescriptor>> {
        for slug in slugs {
            require(
                self.descriptor(slug).is_some(),
                &format!("unknown harness {slug}"),
            )?;
        }
        Ok(self
            .descriptors
            .iter()
            .filter(|d| slugs.is_empty() || slugs.iter().any(|s| s == d.slug()))
            .cloned()
            .collect())
    }
}
impl HarnessDescriptor {
    pub fn slug(&self) -> &str {
        self.as_value()["slug"].as_str().expect("admitted slug")
    }
    pub fn native_kind(&self) -> &str {
        self.as_value()["native_kind"]
            .as_str()
            .expect("admitted kind")
    }
    pub fn probe_plan(&self) -> &serde_json::Map<String, Value> {
        self.as_value()["probe"].as_object().expect("admitted plan")
    }
}
impl BoundaryCatalogue for NativeCatalog {
    fn boundary(&self, harness: &str, native_event: &str) -> Result<NativeBoundary> {
        let slug = harness.strip_prefix("harness/").unwrap_or(harness);
        let descriptor = self
            .capability(slug)
            .ok_or_else(|| Error::new(format!("no declared boundary capability for {harness}")))?;
        let events = descriptor.as_value()["native_events"]
            .as_array()
            .expect("admitted events");
        let entry = events
            .iter()
            .find(|e| e["native_name"] == native_event)
            .ok_or_else(|| {
                Error::new(format!(
                    "undeclared native event {native_event} for {harness}"
                ))
            })?;
        let kind = entry["event"].as_str().expect("admitted event");
        Ok(NativeBoundary {
            harness: actuation_core::ExternalRef::new(harness)?,
            native_event: actuation_core::ExternalRef::new(native_event)?,
            boundary: if kind == "custom" {
                None
            } else {
                Some(actuation_core::ExternalRef::new(kind)?)
            },
            catalog_revision: serde_json::from_value(
                descriptor.as_value()["provenance"]["catalog_revision"].clone(),
            )?,
        })
    }
}
