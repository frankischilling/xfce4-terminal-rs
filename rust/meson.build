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
  '../src/lib.rs',
  '../src/main.rs',
  '../src/reference.rs',
  '../tests/differential_harness.rs',
  '../tests/reference/BASELINE',
  '../tests/reference/Containerfile',
  '../tests/reference/container-build.sh',
  '../tests/reference/differential.sh',
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
