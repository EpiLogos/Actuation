// Committed runner for the self-contained bundle. `scripts/bundle.mjs`
// inlines this import graph (cli/ + contracts/ + detection/) into a single
// `bin/actuation`. This file is only a build input; it is not itself shipped.
import { main } from "../cli/actuation.mjs";

process.exitCode = main();
