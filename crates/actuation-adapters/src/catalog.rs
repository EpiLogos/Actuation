use crate::wire::*;
use crate::{
    Error, HarnessCapability, HarnessCatalogDocument, HarnessDescriptor, Result,
    SecretSourceCatalog,
};
use serde_json::Value;
use std::collections::HashSet;

/// Versioned target facts. Extending the catalogue does not add a harness or
/// model species to generic agency. These are declarations, not live presence.
#[derive(Clone, Debug)]
pub struct Catalog {
    harnesses: HarnessCatalogDocument,
    descriptors: Vec<HarnessDescriptor>,
    capabilities: Vec<HarnessCapability>,
    capability_document: Value,
    secrets: SecretSourceCatalog,
}
impl Catalog {
    pub fn embedded() -> Result<Self> {
        Self::from_documents(
            serde_json::from_str(include_str!("../../../catalog/harnesses.json"))?,
            serde_json::from_str(include_str!("../../../catalog/capabilities.json"))?,
            serde_json::from_str(include_str!("../../../catalog/secret-sources.json"))?,
        )
    }
    pub fn from_documents(harnesses: Value, capabilities: Value, secrets: Value) -> Result<Self> {
        let harnesses = HarnessCatalogDocument::new(harnesses)?;
        let descriptors: Vec<HarnessDescriptor> =
            serde_json::from_value(harnesses.as_value()["descriptors"].clone())?;
        object(&capabilities)?;
        exact(
            &capabilities["schema"],
            "actuation.capability-catalog/v1",
            "schema",
        )?;
        if capabilities["catalog_revision"] != harnesses.as_value()["catalog_revision"] {
            return Err(Error::new(
                "capabilities and target catalogue must share their source revision",
            ));
        }
        let mut seen = HashSet::new();
        let mut admitted = Vec::new();
        for value in list(&capabilities["capabilities"], "capabilities")? {
            let capability = HarnessCapability::new(value.clone())?;
            if !descriptors.iter().any(|d| d.slug() == capability.slug())
                || !seen.insert(capability.slug().to_owned())
            {
                return Err(Error::new("capability must name a unique declared target"));
            }
            admitted.push(capability);
        }
        Ok(Self {
            harnesses,
            descriptors,
            capabilities: admitted,
            capability_document: capabilities,
            secrets: SecretSourceCatalog::new(secrets)?,
        })
    }
    pub fn revision(&self) -> &Value {
        &self.harnesses.as_value()["catalog_revision"]
    }
    pub fn harness_document(&self) -> &HarnessCatalogDocument {
        &self.harnesses
    }
    pub fn capability_document(&self) -> &Value {
        &self.capability_document
    }
    pub fn descriptors(&self) -> &[HarnessDescriptor] {
        &self.descriptors
    }
    pub fn by_slug(&self, slug: &str) -> Option<&HarnessDescriptor> {
        self.descriptors.iter().find(|d| d.slug() == slug)
    }
    pub fn capabilities(&self) -> &[HarnessCapability] {
        &self.capabilities
    }
    pub fn capability_by_slug(&self, slug: &str) -> Option<&HarnessCapability> {
        self.capabilities.iter().find(|c| c.slug() == slug)
    }
    pub fn secret_catalog(&self) -> &SecretSourceCatalog {
        &self.secrets
    }
}

impl actuation_stream::BoundaryCatalogue for Catalog {
    fn boundary(
        &self,
        harness: &str,
        native_event: &str,
    ) -> Result<actuation_stream::NativeBoundary> {
        let capability = self
            .capability_by_slug(harness)
            .ok_or_else(|| Error::new("no capability descriptor for harness"))?;
        let event = capability
            .declared_events()
            .iter()
            .find(|e| e["native_name"] == native_event)
            .ok_or_else(|| Error::new("undeclared native event"))?;
        Ok(actuation_stream::NativeBoundary {
            harness: actuation_core::ExternalRef::new(harness)?,
            native_event: actuation_core::ExternalRef::new(native_event)?,
            boundary: if event["event"] == "custom" {
                None
            } else {
                Some(serde_json::from_value(event["event"].clone())?)
            },
            catalog_revision: serde_json::from_value(
                capability.as_value()["provenance"]["catalog_revision"].clone(),
            )?,
        })
    }
}
