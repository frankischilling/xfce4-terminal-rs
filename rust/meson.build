cargo = find_program('cargo')
cargo_build = find_program('../build-aux/cargo-build.sh')
rust_source_root = meson.current_source_dir() / '..'

rust_profile = 'debug'
if get_option('optimization') not in ['0', 'g']
  rust_profile = 'release'
endif

rust_features = []
if enable_x11
  rust_features += 'x11'
endif
if enable_wayland
  rust_features += 'wayland'
endif
if gtk_layer_shell.found()
  rust_features += 'layer-shell'
endif
if libutempter.found()
  rust_features += 'utempter'
endif
rust_feature_list = ','.join(rust_features)
rust_feature_args = ['--no-default-features']
if rust_features.length() > 0
  rust_feature_args += ['--features', rust_feature_list]
endif
rust_test_profile_args = []
if rust_profile == 'release'
  rust_test_profile_args += '--release'
endif

rust_target_dir = meson.current_build_dir() / 'cargo-target'
rust_sources = files(
  '../Cargo.lock',
  '../Cargo.toml',
  '../rust-toolchain.toml',
  '../src/accelerators.rs',
  '../src/bin/accelerator_probe.rs',
  '../src/bin/link_probe.rs',
  '../src/bin/options_probe.rs',
  '../src/bin/preferences_probe.rs',
  '../src/cli.rs',
  '../src/colors.rs',
  '../src/ffi/glib.rs',
  '../src/ffi/gtk.rs',
  '../src/ffi/libc.rs',
  '../src/ffi/mod.rs',
  '../src/ffi/pcre2.rs',
  '../src/ffi/xfce.rs',
  '../src/ffi/xfconf.rs',
  '../src/lib.rs',
  '../src/links.rs',
  '../src/localization.rs',
  '../src/main.rs',
  '../src/preferences.rs',
  '../src/reference.rs',
  '../tests/preference_assets.rs',
  '../tests/accelerator_round_trip.rs',
  '../tests/link_matching.rs',
  '../tests/preferences_parity.rs',
  '../tests/xfconf_round_trip.rs',
  '../tests/support/mod.rs',
  '../tests/differential_harness.rs',
  '../tests/cli_parity.rs',
  '../tests/cli_process.rs',
  '../tests/reference/BASELINE',
  '../tests/reference/accelerator-contract.tsv',
  '../tests/reference/Containerfile',
  '../tests/reference/build-options-probe.sh',
  '../tests/reference/build-application-call-probe.sh',
  '../tests/reference/build-link-probe.sh',
  '../tests/reference/build-preferences-probe.sh',
  '../tests/reference/application-call-probe.c',
  '../tests/reference/application-calls.sh',
  '../tests/reference/container-build.sh',
  '../tests/reference/differential.sh',
  '../tests/reference/link-contract.tsv',
  '../tests/reference/link-fixtures.txt',
  '../tests/reference/link-matching.sh',
  '../tests/reference/link-probe.c',
  '../tests/reference/options-probe.c',
  '../tests/reference/preferences-behavior.sh',
  '../tests/reference/preferences-contract.tsv',
  '../tests/reference/preferences-probe.c',
  '../tests/reference/probe-command.py',
  '../tests/reference_contract.rs',
)

custom_target(
  'xfce4-terminal-rs',
  input: rust_sources,
  output: 'xfce4-terminal-rs',
  command: [
    cargo_build,
    cargo,
    rust_source_root,
    rust_target_dir,
    '@OUTPUT@',
    rust_profile,
    rust_feature_list,
    get_option('debug') ? 'true' : 'false',
    get_option('prefix') / get_option('localedir'),
  ],
  build_by_default: true,
  build_always_stale: true,
)

test(
  'rust',
  cargo,
  args: [
    'test',
    '--manifest-path', rust_source_root / 'Cargo.toml',
    '--target-dir', rust_target_dir,
    '--all-targets',
    '--locked',
  ] + rust_feature_args + rust_test_profile_args,
  timeout: 300,
)
