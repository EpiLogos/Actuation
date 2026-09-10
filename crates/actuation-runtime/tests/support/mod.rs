use actuation_core::{Error, Result};
use actuation_runtime::*;
use serde_json::{json, Value};

pub fn evaluate(operation: &str, args: &[Value]) -> Result<Value> {
    let first = args.first().cloned().unwrap_or(Value::Null);
    match operation {
        "contracts/agency-actualisation.mjs#actualiseAgency" => {
            Ok(serde_json::from_value::<ActualisationRequest>(first)?
                .admit()?
                .receipt())
        }
        "contracts/realised-actuation.mjs#validateRealisedActuation" => {
            Ok(serde_json::to_value(serde_json::from_value::<
                RealisedActuation,
            >(first)?)?)
        }
        "contracts/realised-actuation.mjs#realisedActuationReadModel" => {
            Ok(serde_json::from_value::<RealisedActuation>(first)?.reading())
        }
        "contracts/realised-actuation.mjs#continuityDelta" => {
            let previous: RealisedActuation = serde_json::from_value(first)?;
            let next: RealisedActuation =
                serde_json::from_value(args.get(1).cloned().unwrap_or(Value::Null))?;
            Ok(json!(previous.continuity_to(&next)))
        }
        _ => Err(Error::new(format!(
            "unsupported runtime oracle operation {operation}"
        ))),
    }
}
