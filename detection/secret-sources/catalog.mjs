// The declared secret-source catalog for actuation.secret-detection/v1.
// Adding a source is one descriptor here plus a CATALOG_REVISION bump. The
// catalog declares only what today's probes can verify — the keychain-entry
// source kind is defined in the contract but has no declared source yet;
// it lands with the native keychain-adapter (central-security map T3), at
// which point the op-service-account-token descriptor flips from env-var to
// keychain-entry and the scan starts reporting env presence as drift.
import { secretSourceDescriptor } from "../../contracts/secret-detection.mjs";

export const SECRET_SOURCE_CATALOG_REVISION = 1;

const DESCRIPTORS = [
  {
    schema: "actuation.secret-detection/v1",
    document: "descriptor",
    slug: "op-vault-security-protocol",
    source_kind: "op-item",
    ref_schemes: ["op://"],
    probe: { "vault-item": { item_ref: "op://Central/central-security" } },
    provenance: {
      authored_by: "central-security map T1",
      source_refs: ["https://github.com/EpiLogos/central-security/issues/1"],
      catalog_revision: SECRET_SOURCE_CATALOG_REVISION,
    },
  },
  {
    schema: "actuation.secret-detection/v1",
    document: "descriptor",
    slug: "op-service-account-token",
    source_kind: "env-var",
    ref_schemes: ["env://"],
    probe: { env: { names: ["OP_SERVICE_ACCOUNT_TOKEN"] } },
    provenance: {
      authored_by: "central-security map T1",
      source_refs: ["https://github.com/EpiLogos/central-security/issues/1"],
      catalog_revision: SECRET_SOURCE_CATALOG_REVISION,
    },
  },
  {
    schema: "actuation.secret-detection/v1",
    document: "descriptor",
    slug: "varlock-env-files",
    source_kind: "varlock-blob",
    ref_schemes: ["varlock://"],
    probe: {
      "file-pattern": {
        patterns: [".env", ".env.local", ".env.*"],
        roots: ["~"],
      },
    },
    provenance: {
      authored_by: "central-security map T1",
      source_refs: ["https://github.com/EpiLogos/central-security/issues/1"],
      catalog_revision: SECRET_SOURCE_CATALOG_REVISION,
    },
  },
].map(secretSourceDescriptor);

export function secretSourceDescriptors() {
  return structuredClone(DESCRIPTORS);
}

export function secretSourceDescriptorBySlug(slug) {
  return structuredClone(DESCRIPTORS.find((descriptor) => descriptor.slug === slug));
}

export function secretSourceCatalog() {
  return {
    schema: "actuation.secret-detection/v1",
    document: "catalog",
    catalog_revision: SECRET_SOURCE_CATALOG_REVISION,
    descriptors: structuredClone(DESCRIPTORS),
  };
}
