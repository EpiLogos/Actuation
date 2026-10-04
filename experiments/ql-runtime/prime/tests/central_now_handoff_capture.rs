//! Optional joined test caller of AIKit's existing native capture owner.
//! This is compiled as an explicitly attributed AIKit example overlay. It is
//! not an Actuation production dependency or the SDK's process implementation.

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod gate {
    use aikit_adapters::runner::SystemRunner;
    use serde::Serialize;
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::os::unix::fs::MetadataExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    type Result<T> = std::result::Result<T, Box<dyn Error>>;
    const CENTRAL: &str = "a3bd680619de0743d4faa313aa1932c8375abc95";
    const LEGACY: &str = "5e4510a6bd61d6e151c84d755f884b61b693db27";
    const SDK: &str = "01d2df70d8e0cf8718be213f0ef0bd35fc793a85e4423183bc9af6252e7f1e75";
    const DRIVER: &str = "48c86f8bc9a778911266615fd6ed0e09e11a3bc347b5fe27abf394c4086276e7";
    const CASES: [&str; 7] = [
        "root_roundtrip_current_native_source",
        "project_member_and_old_owner_compatibility",
        "exact_selected_source_preserves_sibling",
        "current_owner_withdrawal_and_unsafe_forms",
        "actual_nonroot_eacces_then_reopen",
        "old_root_without_route_is_unavailable_not_resubmitted",
        "real_selected_malformed_and_budget_material",
    ];

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    struct Basis {
        device: u64,
        inode: u64,
        mode: u32,
        links: u64,
        length: u64,
        modified: (i64, i64),
        changed: (i64, i64),
    }
    impl Basis {
        fn of(value: &fs::Metadata) -> Self {
            Self {
                device: value.dev(),
                inode: value.ino(),
                mode: value.mode(),
                links: value.nlink(),
                length: value.len(),
                modified: (value.mtime(), value.mtime_nsec()),
                changed: (value.ctime(), value.ctime_nsec()),
            }
        }
    }
    fn require(value: bool, message: &'static str) -> Result<()> {
        if !value {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, message).into());
        }
        Ok(())
    }
    fn open(path: &Path, directory: bool) -> Result<File> {
        use rustix::fs::{Mode, OFlags};
        let mut flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
        if directory {
            flags |= OFlags::DIRECTORY;
        }
        Ok(File::from(rustix::fs::open(path, flags, Mode::empty())?))
    }
    struct Owner {
        path: PathBuf,
        held: File,
        identity: (u64, u64),
    }
    impl Owner {
        fn capture(path: PathBuf, empty: bool) -> Result<Self> {
            require(path.is_absolute() && path.canonicalize()? == path, "ordinary root required")?;
            let held = open(&path, true)?;
            let value = held.metadata()?;
            require(value.is_dir(), "native evidence root must be a directory")?;
            let owner = Self { path, held, identity: (value.dev(), value.ino()) };
            owner.validate()?;
            if empty {
                require(fs::read_dir(&owner.path)?.next().is_none(), "exclusive empty root required")?;
            }
            Ok(owner)
        }
        fn validate(&self) -> Result<()> {
            let named = fs::symlink_metadata(&self.path)?;
            let held = self.held.metadata()?;
            require(
                named.is_dir() && held.is_dir()
                    && (named.dev(), named.ino()) == self.identity
                    && (held.dev(), held.ino()) == self.identity
                    && self.path.canonicalize()? == self.path,
                "evidence owner affiliation changed",
            )
        }
        fn write(&self, name: &str, bytes: &[u8]) -> Result<()> {
            use rustix::fs::{Mode, OFlags};
            require(!name.contains('/') && !name.starts_with('.'), "normal evidence member required")?;
            self.validate()?;
            let fd = rustix::fs::openat(
                &self.held,
                name,
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o600),
            )?;
            let mut file = File::from(fd);
            file.write_all(bytes)?;
            file.sync_all()?;
            self.validate()?;
            Ok(())
        }
        fn json(&self, name: &str, value: &Value) -> Result<()> {
            self.write(name, &serde_json::to_vec_pretty(value)?)
        }
    }
    struct Input {
        path: PathBuf,
        held: File,
        basis: Basis,
        digest: String,
        seconds: f64,
        bytes: Vec<u8>,
    }
    impl Input {
        fn capture(path: PathBuf, limit: u64) -> Result<Self> {
            require(path.is_absolute() && path.canonicalize()? == path, "ordinary input required")?;
            let named = fs::symlink_metadata(&path)?;
            require(named.is_file() && named.nlink() >= 1, "regular native input required")?;
            let mut held = open(&path, false)?;
            let basis = Basis::of(&named);
            require(Basis::of(&held.metadata()?) == basis && basis.length <= limit, "input changed or oversized")?;
            let start = Instant::now();
            let mut bytes = Vec::new();
            (&mut held).take(limit + 1).read_to_end(&mut bytes)?;
            require(bytes.len() as u64 <= limit, "input budget exhausted")?;
            let digest = format!("{:x}", Sha256::digest(&bytes));
            let value = Self { path, held, basis, digest, seconds: start.elapsed().as_secs_f64(), bytes };
            value.validate()?;
            Ok(value)
        }
        fn validate(&self) -> Result<()> {
            require(
                Basis::of(&self.held.metadata()?) == self.basis
                    && Basis::of(&fs::symlink_metadata(&self.path)?) == self.basis
                    && self.path.canonicalize()? == self.path,
                "held or named required input changed",
            )
        }
        fn observation(&self) -> Value {
            json!({"path":self.path,"basis":self.basis,"sha256":self.digest,
                "sha2_cost":{"bytes":self.basis.length,"seconds":self.seconds}})
        }
        fn release_body(&mut self) {
            self.bytes.clear();
            self.bytes.shrink_to_fit();
        }
    }
    fn path(value: &Value, key: &str) -> Result<PathBuf> {
        value[key].as_str().map(PathBuf::from).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "required coordinate absent").into()
        })
    }
    fn check_lines(stdout: &str) -> Result<Value> {
        let values: Vec<Value> = stdout.lines().map(serde_json::from_str).collect::<std::result::Result<_, _>>()?;
        require(values.len() == 8, "actual seven results and one summary required")?;
        for (value, case) in values.iter().zip(CASES) {
            require(value["case"] == case && value["result"] == "passed", "actual case identity or outcome differs")?;
            require(value["seconds"].as_f64().is_some_and(|n| n.is_finite() && n >= 0.0), "actual case time required")?;
        }
        let summary = &values[7];
        require(summary["selected"] == 7 && summary["passed"] == 7 && summary["skipped"] == 0, "seven executed bodies required")?;
        require(summary["native_calls"].as_u64().is_some_and(|n| n > 0), "actual native calls required")?;
        Ok(json!(values))
    }
    pub fn run() -> Result<()> {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        require(args.len() == 1, "one native gate configuration required")?;
        let config = Input::capture(PathBuf::from(&args[0]), 65_536)?;
        let value: Value = serde_json::from_slice(&config.bytes)?;
        require(value["schema"] == "actuation.central-handoff-capture-config/v1", "gate schema mismatch")?;
        let owner = Owner::capture(path(&value, "evidence_root")?, true)?;
        let fixtures = Owner::capture(path(&value, "artifact_root")?, true)?;
        let driver = Input::capture(path(&value, "driver")?, 1_048_576)?;
        let sdk = Input::capture(path(&value, "sdk_source")?, 1_048_576)?;
        require(driver.digest == DRIVER && sdk.digest == SDK, "exact selected consumer source required")?;
        let mut python = Input::capture(path(&value, "python")?, 256 * 1024 * 1024)?;
        require(python.basis.mode & 0o111 != 0, "actual executable Python required")?;
        python.release_body();
        let mut inputs = vec![config, driver, sdk, python];
        for (key, role, revision) in [("current_build", "current", CENTRAL), ("legacy_build", "legacy", LEGACY)] {
            let manifest = Input::capture(path(&value, key)?, 1_048_576)?;
            let record: Value = serde_json::from_slice(&manifest.bytes)?;
            require(record["role"] == role && record["source_revision"] == revision, "actual owner source role mismatch")?;
            let mut image = Input::capture(path(&record, "executable")?, 256 * 1024 * 1024)?;
            require(record["sha256"] == image.digest && image.basis.mode & 0o111 != 0, "compiled native image differs")?;
            image.release_body();
            inputs.extend([manifest, image]);
        }
        let mut this = Input::capture(std::env::current_exe()?.canonicalize()?, 256 * 1024 * 1024)?;
        this.release_body();
        inputs.push(this);
        owner.json("before-operation.json", &json!({"schema":"actuation.central-handoff-native-custody/v1",
            "source_capture_owner":"900bce05483c6fd39cfd320ebf95c2e2b5aa4d29",
            "inputs":inputs.iter().map(Input::observation).collect::<Vec<_>>(),
            "fixture_root":fixtures.path,"fixture_identity":fixtures.identity,
            "disposition":"retained before execution; no automatic fixture removal",
            "live_allowance_ms":180_000,"retirement_allowance_ms":2_000,
            "raw_per_stream_limit":524_288,"required_cases":CASES}))?;
        let mut command = Command::new(&inputs[3].path);
        command.arg(&inputs[1].path)
            .arg("--current-build").arg(path(&value, "current_build")?)
            .arg("--legacy-build").arg(path(&value, "legacy_build")?)
            .arg("--artifact-root").arg(&fixtures.path)
            .arg("--sdk-source").arg(&inputs[2].path)
            .arg("--sdk-sha256").arg(SDK)
            .current_dir(&fixtures.path).env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("TMPDIR", &fixtures.path).env("PYTHONDONTWRITEBYTECODE", "1")
            .stdin(Stdio::null());
        for input in &inputs {
            input.validate()?;
        }
        fixtures.validate()?;
        owner.validate()?;
        let runner = SystemRunner::new()
            .with_timeout(Duration::from_secs(180))
            .with_output_limit_bytes(512 * 1024)
            .with_strict_utf8()
            .with_body_free_diagnostics()
            .with_unix_signal_status();
        let start = Instant::now();
        let result = runner.capture_command(&mut command);
        owner.validate()?;
        let observation = match &result {
            Ok(output) => {
                owner.write("driver.stdout", output.stdout.as_bytes())?;
                owner.write("driver.stderr", output.stderr.as_bytes())?;
                json!({"status":output.status,"elapsed_seconds":start.elapsed().as_secs_f64(),
                    "source_contract":"strict capture Ok; no invented per-pipe facts",
                    "inner_fixture_retirement":"not inferred from outer process completion"})
            }
            Err(error) => {
                if let Some(capture) = error.native_capture() {
                    owner.write("driver.partial.stdout", &capture.stdout)?;
                    owner.write("driver.partial.stderr", &capture.stderr)?;
                }
                let io = |cause: &std::io::Error| json!({"kind":format!("{:?}",cause.kind()),"raw_os_error":cause.raw_os_error()});
                let primary = error.source().and_then(|cause| cause.downcast_ref::<std::io::Error>()).map(io);
                let secondary: Vec<_> = error.secondary_io_sources().map(io).collect();
                json!({"code":error.code(),"details":error.details(),"primary_io":primary,
                    "secondary_io":secondary,"elapsed_seconds":start.elapsed().as_secs_f64(),
                    "capture_status":error.native_capture().and_then(|capture|capture.status),
                    "disposition":"failed; partial evidence retained; no automatic resend or sweep"})
            }
        };
        owner.json("native-result.json", &observation)?;
        for input in &inputs {
            input.validate()?;
        }
        fixtures.validate()?;
        let output = result?;
        require(output.status == 0, "actual joined driver failed")?;
        let cases = check_lines(&output.stdout)?;
        owner.json("qualification.json", &json!({"actual_driver_cases":cases,
            "selected":7,"passed":7,"skipped":0,"fixture_disposition":"retained",
            "sdk_production_capture":"not repaired by this optional test harness",
            "scope":"Source-bound hosted native owner/SDK fixtures; no installed/provider/Original credit"}))?;
        println!("seven actual native handoff cases passed; retained owner evidence required");
        Ok(())
    }
}

fn main() {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    if let Err(error) = gate::run() {
        let cause = error.downcast_ref::<std::io::Error>()
            .map(|value| (value.kind(), value.raw_os_error()));
        eprintln!("native handoff gate failed; original IO={cause:?}; retained evidence required");
        std::process::exit(1);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        eprintln!("native handoff gate requires supported Linux/Mac capture");
        std::process::exit(1);
    }
}
