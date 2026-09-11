//! QL is supplied by an exact owner instrument. No formal laws live here.
use crate::{evidence::bytes_digest, prime::full_revision, process::ProcessSpec, Error, Result};
use serde_json::{json, Value};
use std::{fs::File, io::Read};
#[derive(Clone, Debug)]
pub struct OwnerInstrument {
    spec: ProcessSpec,
    revision: String,
    binary_digest: String,
}
impl OwnerInstrument {
    pub fn bind(spec: ProcessSpec, revision: &str) -> Result<Self> {
        full_revision(&json!(revision), "QL owner revision")?;
        spec.validate()?;
        let digest = Self::digest(&spec)?;
        let owner = Self {
            spec,
            revision: revision.into(),
            binary_digest: digest,
        };
        let c = owner.invoke(json!({"operation":"capabilities"}))?;
        if c["result"]["formal_owner"] != "EpiLogos/QL-MEF" {
            return Err(Error::new("instrument did not disclose QL formal owner"));
        }
        Ok(owner)
    }
    fn digest(spec: &ProcessSpec) -> Result<String> {
        let file =
            File::open(&spec.program).map_err(|_| Error::new("owner instrument unavailable"))?;
        if !file
            .metadata()
            .map_err(|_| Error::new("owner metadata unavailable"))?
            .is_file()
        {
            return Err(Error::new("owner instrument is not regular"));
        }
        let mut bytes = Vec::new();
        file.take(128 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::new("owner digest read failed"))?;
        if bytes.len() > 128 * 1024 * 1024 {
            return Err(Error::new("owner instrument exceeds digest bound"));
        }
        Ok(bytes_digest(&bytes))
    }
    pub fn invoke(&self, request: Value) -> Result<Value> {
        if Self::digest(&self.spec)? != self.binary_digest {
            return Err(Error::new("bound owner instrument bytes changed"));
        }
        let operation = request["operation"]
            .as_str()
            .ok_or_else(|| Error::new("owner operation required"))?;
        let r = self.spec.run(
            &serde_json::to_vec(&request).map_err(|_| Error::new("owner request encode failed"))?,
        )?;
        if r.code != Some(0) {
            return Err(Error::new(
                "QL owner refused operation; no local formal fallback",
            ));
        }
        let v: Value =
            serde_json::from_str(&r.stdout).map_err(|_| Error::new("invalid owner reply"))?;
        if v["schema"] != "actuation.ql-owner-operation/v1"
            || v["owner_repository"] != "EpiLogos/QL-MEF"
            || v["owner_revision"] != self.revision
            || v["operation"] != operation
            || v.get("result").is_none()
        {
            return Err(Error::new("owner response correlation/revision mismatch"));
        }
        if Self::digest(&self.spec)? != self.binary_digest {
            return Err(Error::new("owner instrument changed during invocation"));
        }
        Ok(v)
    }
    pub fn basis(&self) -> Value {
        json!({"repository":"EpiLogos/QL-MEF","revision":self.revision,"instrument_sha256":self.binary_digest})
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    const TEST_REVISION: &str = "e753efc91f62b5b2af09e0a852c5063e366eccbe";

    fn instrument_script(extra: &str) -> String {
        format!(
            r#"#!/bin/sh
request=$(cat)
operation=$(printf '%s' "$request" | /usr/bin/sed -n 's/.*"operation":"\([^"]*\)".*/\1/p')
if [ "$operation" = "capabilities" ]; then
  printf '{{"schema":"actuation.ql-owner-operation/v1","owner_repository":"EpiLogos/QL-MEF","owner_revision":"{TEST_REVISION}","operation":"capabilities","result":{{"formal_owner":"EpiLogos/QL-MEF","positions":6}}}}\n'
else
  {extra}
fi
"#
        )
    }

    fn spec_for(script: &str) -> (tempfile::TempDir, ProcessSpec) {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("owner-instrument");
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let spec = ProcessSpec {
            program: path,
            args: vec![],
            cwd: dir.path().to_owned(),
            environment: Default::default(),
            timeout_ms: 10_000,
            output_limit: 1 << 20,
        };
        (dir, spec)
    }

    #[test]
    fn bind_accepts_only_a_disclosing_owner_instrument() {
        let (_dir, spec) = spec_for(&instrument_script("exit 1"));
        let owner = OwnerInstrument::bind(spec, TEST_REVISION).expect("instrument binds");
        assert_eq!(owner.basis()["repository"], json!("EpiLogos/QL-MEF"));
        assert_eq!(owner.basis()["revision"], json!(TEST_REVISION));
        let (_dir, spec) = spec_for(&instrument_script("exit 1"));
        assert!(
            OwnerInstrument::bind(spec, "short").is_err(),
            "bad revision"
        );
    }

    #[test]
    fn invoke_correlates_the_reply_and_detects_byte_changes() {
        let (_dir, spec) = spec_for(&instrument_script("exit 1"));
        let owner = OwnerInstrument::bind(spec, TEST_REVISION).unwrap();
        // A refused operation is an error, never a local formal fallback.
        assert!(owner.invoke(json!({"operation":"vocabulary"})).is_err());
        // An uncorrelated reply is refused.
        let (_dir, spec) = spec_for(&instrument_script(
            r#"printf '{"schema":"actuation.ql-owner-operation/v1","owner_repository":"EpiLogos/QL-MEF","owner_revision":"e753efc91f62b5b2af09e0a852c5063e366eccbe","operation":"mismatched","result":{}}\n'"#,
        ));
        let owner = OwnerInstrument::bind(spec, TEST_REVISION).unwrap();
        assert!(owner.invoke(json!({"operation":"vocabulary"})).is_err());

        // A correct scripted reply for the operation passes correlation.
        let (dir, spec) = spec_for(&instrument_script(
            r#"printf '{"schema":"actuation.ql-owner-operation/v1","owner_repository":"EpiLogos/QL-MEF","owner_revision":"e753efc91f62b5b2af09e0a852c5063e366eccbe","operation":"vocabulary","result":{"positions":6}}\n'"#,
        ));
        let owner = OwnerInstrument::bind(spec.clone(), TEST_REVISION).unwrap();
        let reply = owner
            .invoke(json!({"operation":"vocabulary"}))
            .expect("correlated reply");
        assert_eq!(reply["result"]["positions"], json!(6));
        // Rewriting the instrument bytes invalidates further invocation.
        std::fs::write(&spec.program, b"#!/bin/sh\nexit 0\n").unwrap();
        let err = owner.invoke(json!({"operation":"vocabulary"})).unwrap_err();
        assert!(err.to_string().contains("bytes changed"));
        drop(dir);
    }
}
