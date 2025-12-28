# Build Log - AccuScene Enterprise v0.1.5

## Build Status: FAIL

## Timestamp: 2025-12-28 20:53:28 UTC

---

## Cargo Check Results

**Exit Code:** 101 (FAILED)

```
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /home/user/accident-recreate/rust-core/crates/accuscene-ffi/Cargo.toml
workspace: /home/user/accident-recreate/rust-core/Cargo.toml
    Updating crates.io index
error: failed to select a version for `libsqlite3-sys`.
    ... required by package `rusqlite v0.30.0`
    ... which satisfies dependency `rusqlite = "^0.30"` of package `accuscene-jobs v0.1.5 (/home/user/accident-recreate/rust-core/crates/accuscene-jobs)`
versions that meet the requirements `^0.27.0` are: 0.27.0

package `libsqlite3-sys` links to the native library `sqlite3`, but it conflicts with a previous package which links to `sqlite3` as well:
package `libsqlite3-sys v0.28.0`
    ... which satisfies dependency `libsqlite3-sys = "^0.28.0"` of package `rusqlite v0.31.0`
    ... which satisfies dependency `rusqlite = "^0.31"` of package `accuscene-database v0.1.5 (/home/user/accident-recreate/rust-core/crates/accuscene-database)`
Only one package in the dependency graph may specify the same links value. This helps ensure that only one copy of a native library is linked in the final binary. Try to adjust your dependencies so that only one package uses the `links = "sqlite3"` value. For more information, see https://doc.rust-lang.org/cargo/reference/resolver.html#links.

failed to select a version for `libsqlite3-sys` which could resolve this conflict
```

---

## Cargo Build Results

**Exit Code:** 101 (FAILED)

```
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /home/user/accident-recreate/rust-core/crates/accuscene-ffi/Cargo.toml
workspace: /home/user/accident-recreate/rust-core/Cargo.toml
    Updating crates.io index
error: failed to select a version for `libsqlite3-sys`.
    ... required by package `rusqlite v0.30.0`
    ... which satisfies dependency `rusqlite = "^0.30"` of package `accuscene-jobs v0.1.5 (/home/user/accident-recreate/rust-core/crates/accuscene-jobs)`
versions that meet the requirements `^0.27.0` are: 0.27.0

package `libsqlite3-sys` links to the native library `sqlite3`, but it conflicts with a previous package which links to `sqlite3` as well:
package `libsqlite3-sys v0.28.0`
    ... which satisfies dependency `libsqlite3-sys = "^0.28.0"` of package `rusqlite v0.31.0`
    ... which satisfies dependency `rusqlite = "^0.31"` of package `accuscene-database v0.1.5 (/home/user/accident-recreate/rust-core/crates/accuscene-database)`
Only one package in the dependency graph may specify the same links value. This helps ensure that only one copy of a native library is linked in the final binary. Try to adjust your dependencies so that only one package uses the `links = "sqlite3"` value. For more information, see https://doc.rust-lang.org/cargo/reference/resolver.html#links.

failed to select a version for `libsqlite3-sys` which could resolve this conflict
```

---

## Clippy Results

**Exit Code:** 101 (FAILED)

```
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /home/user/accident-recreate/rust-core/crates/accuscene-ffi/Cargo.toml
workspace: /home/user/accident-recreate/rust-core/Cargo.toml
    Updating crates.io index
error: failed to select a version for `libsqlite3-sys`.
    ... required by package `rusqlite v0.30.0`
    ... which satisfies dependency `rusqlite = "^0.30"` of package `accuscene-jobs v0.1.5 (/home/user/accident-recreate/rust-core/crates/accuscene-jobs)`
versions that meet the requirements `^0.27.0` are: 0.27.0

package `libsqlite3-sys` links to the native library `sqlite3`, but it conflicts with a previous package which links to `sqlite3` as well:
package `libsqlite3-sys v0.28.0`
    ... which satisfies dependency `libsqlite3-sys = "^0.28.0"` of package `rusqlite v0.31.0`
    ... which satisfies dependency `rusqlite = "^0.31"` of package `accuscene-database v0.1.5 (/home/user/accident-recreate/rust-core/crates/accuscene-database)`
Only one package in the dependency graph may specify the same links value. This helps ensure that only one copy of a native library is linked in the final binary. Try to adjust your dependencies so that only one package uses the `links = "sqlite3"` value. For more information, see https://doc.rust-lang.org/cargo/reference/resolver.html#links.

failed to select a version for `libsqlite3-sys` which could resolve this conflict
```

---

## Summary

- **Total crates:** 11
- **Successfully built:** 0
- **Errors:** 1 (dependency conflict)
- **Warnings:** 1 (profile configuration)

### Critical Error Details

**Error Type:** Dependency Conflict - Native Library Linking

**Root Cause:** Version mismatch in `rusqlite` dependency across workspace crates:
- `accuscene-jobs` uses `rusqlite = "^0.30"` (depends on `libsqlite3-sys ^0.27.0`)
- `accuscene-database` uses `rusqlite = "^0.31"` (depends on `libsqlite3-sys ^0.28.0`)

**Impact:** Both versions attempt to link to the same native library `sqlite3`, which Cargo does not allow. Only one version of a native library can be linked in the final binary.

**Resolution Required:** Align `rusqlite` versions across all workspace crates. Recommended action:
1. Update `accuscene-jobs/Cargo.toml` to use `rusqlite = "^0.31"` (matching `accuscene-database`)
2. OR update `accuscene-database/Cargo.toml` to use `rusqlite = "^0.30"` (matching `accuscene-jobs`)
3. Recommended: Use `rusqlite = "^0.31"` for both (latest version)

### Additional Warning

**Warning Type:** Profile Configuration

**Details:** Profile settings in `/home/user/accident-recreate/rust-core/crates/accuscene-ffi/Cargo.toml` will be ignored. Profiles should only be specified at the workspace root.

**Resolution Required:** Move any profile settings from `accuscene-ffi/Cargo.toml` to `/home/user/accident-recreate/rust-core/Cargo.toml`

---

## Build Agent Notes

Build process completely blocked by dependency resolution failure. No crates could be checked, built, or linted until the rusqlite version conflict is resolved.

**Next Steps for Build Errors Agent:**
1. Fix the `rusqlite` version mismatch in crate dependencies
2. Remove profile configurations from individual crate manifests
3. Re-run build process after fixes are applied

---

**Build Agent:** COMPLETED MONITORING CYCLE
**Status:** AWAITING ERROR RESOLUTION
