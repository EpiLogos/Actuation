use actuation_core::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

fn admitted<T: DeserializeOwned + Serialize>(input: Value) -> Result<Value> {
    let value: T = serde_json::from_value(input)?;
    Ok(serde_json::to_value(value)?)
}
// Temporary R1 transport only. All work is delegated to ordinary domain APIs.
pub fn evaluate(operation: &str, args: &[Value]) -> Result<Value> {
    let first = args.first().cloned().unwrap_or(Value::Null);
    match operation.strip_prefix("contracts/agency.mjs#") {
        Some("validateWorldBinding") => admitted::<WorldBinding>(first),
        Some("validateRootScope") => admitted::<RootScope>(first),
        Some("validateMetagencyGrant") => admitted::<MetagencyGrant>(first),
        Some("validateDetermination") => admitted::<Determination>(first),
        Some("validateReturn") => admitted::<Return>(first),
        Some("validateDeterminationLineage") => admitted::<DeterminationLineage>(first),
        Some("isRootAgency") => {
            let binding: WorldBinding = serde_json::from_value(first)?;
            let scope: RootScope =
                serde_json::from_value(args.get(1).cloned().unwrap_or(Value::Null))?;
            Ok(json!(binding.is_root_for(&scope)))
        }
        Some("agencyReadModel") => Ok(serde_json::to_value(agency_reading(first)?)?),
        _ => Err(Error::new(format!(
            "unsupported constitutional oracle operation {operation}"
        ))),
    }
}
