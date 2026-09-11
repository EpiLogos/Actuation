// The frozen Wave 5 System disclosure identity. Kept in its own leaf module so
// both the CLI surface (which advertises the contract version) and the
// disclosure builder (which emits the descriptor) can import it without
// creating a module cycle.
export const SYSTEM_DISCLOSURE_VERSION = "oi.product-settings-disclosure/v2";
export const SYSTEM_DISCLOSURE_CONTRACT_REVISION = "wave-5/system.1";
