//! Typed preference definitions shared by storage, migration, and UI code.

use std::sync::OnceLock;
use std::{env, path::PathBuf};

use crate::ffi::{libc, xfconf};

const DEFINITION_CONTRACT: &str = include_str!("../tests/reference/preferences-contract.tsv");

/// The storage type and validation domain of one preference.
#[derive(Clone, Debug, PartialEq)]
pub enum PreferenceKind {
    Boolean,
    String,
    Unsigned {
        minimum: u32,
        maximum: u32,
    },
    Double {
        minimum: f64,
        maximum: f64,
    },
    Enumeration {
        type_name: &'static str,
        values: Vec<&'static str>,
    },
}

/// One entry in the Xfce Terminal preference schema.
#[derive(Clone, Debug, PartialEq)]
pub struct PreferenceDefinition {
    pub name: &'static str,
    pub kind: PreferenceKind,
    pub default: Option<&'static str>,
    pub legacy_key: &'static str,
}

impl PreferenceDefinition {
    /// Returns this definition's frozen C default as a typed value.
    pub fn default_value(&self) -> PreferenceValue {
        default_value(self)
    }
}

/// A preference value in the representation stored by Xfconf.
#[derive(Clone, Debug, PartialEq)]
pub enum PreferenceValue {
    Boolean(bool),
    String(Option<String>),
    Unsigned(u32),
    Double(f64),
    Enumeration(String),
}

/// Synchronous access to one Xfconf preference channel.
pub struct Preferences {
    session: xfconf::Session,
}

/// An invalid preference request or a native Xfconf failure.
#[derive(Debug, PartialEq)]
pub struct PreferenceError(String);

impl std::fmt::Display for PreferenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for PreferenceError {}

impl Preferences {
    /// Opens a channel after initializing libxfconf for this instance.
    pub fn new(channel_name: &str) -> Result<Self, PreferenceError> {
        xfconf::Session::new(channel_name)
            .map(|session| Self { session })
            .map_err(PreferenceError)
    }

    /// Reads a value from Xfconf or returns the frozen C default.
    pub fn get(&self, name: &str) -> Result<PreferenceValue, PreferenceError> {
        let definition = required_definition(name)?;
        if !self.session.has(name).map_err(PreferenceError)? {
            return Ok(default_value(definition));
        }

        match &definition.kind {
            PreferenceKind::Boolean => self.session.get_bool(name).map(PreferenceValue::Boolean),
            PreferenceKind::String => self
                .session
                .get_string(name)
                .map(|value| PreferenceValue::String(Some(value))),
            PreferenceKind::Unsigned { .. } => {
                self.session.get_uint(name).map(PreferenceValue::Unsigned)
            }
            PreferenceKind::Double { .. } => {
                self.session.get_double(name).map(PreferenceValue::Double)
            }
            PreferenceKind::Enumeration { .. } => self
                .session
                .get_string(name)
                .map(PreferenceValue::Enumeration),
        }
        .map_err(PreferenceError)
    }

    /// Validates and writes one value using the C application's storage type.
    pub fn set(&self, name: &str, value: PreferenceValue) -> Result<(), PreferenceError> {
        let definition = required_definition(name)?;
        validate(definition, &value)?;

        match value {
            PreferenceValue::Boolean(value) => self.session.set_bool(name, value),
            PreferenceValue::String(Some(value)) => self.session.set_string(name, &value),
            PreferenceValue::String(None) => self.session.reset(name),
            PreferenceValue::Unsigned(value) => self.session.set_uint(name, value),
            PreferenceValue::Double(value) => self.session.set_double(name, value),
            PreferenceValue::Enumeration(value) => self.session.set_string(name, &value),
        }
        .map_err(PreferenceError)
    }

    /// Removes a stored value so later reads use the default.
    pub fn reset(&self, name: &str) -> Result<(), PreferenceError> {
        required_definition(name)?;
        self.session.reset(name).map_err(PreferenceError)
    }

    /// Imports the first legacy terminalrc found when the channel is new.
    pub fn migrate_legacy(&self) -> Result<usize, PreferenceError> {
        if self.session.channel_existed() {
            return Ok(0);
        }

        let config_home = config_home()?;
        let current = config_home.join("xfce4/terminal/terminalrc");
        let old = config_home.join("Terminal/terminalrc");
        let (path, migrate_palette) = if current.is_file() {
            (current, false)
        } else if old.is_file() {
            (old, true)
        } else {
            return Ok(0);
        };

        let key_file = glib::KeyFile::new();
        key_file
            .load_from_file(path, glib::KeyFileFlags::NONE)
            .map_err(|error| PreferenceError(error.to_string()))?;

        let mut migrated = 0;
        for definition in definitions() {
            if !key_file
                .has_key("Configuration", definition.legacy_key)
                .unwrap_or(false)
            {
                continue;
            }
            let source = key_file
                .string("Configuration", definition.legacy_key)
                .map_err(|error| PreferenceError(error.to_string()))?;
            self.set(definition.name, legacy_value(definition, source.as_str()))?;
            migrated += 1;
        }

        if migrate_palette {
            let colors = (1..=16)
                .map(|index| {
                    key_file
                        .string("Configuration", &format!("ColorPalette{index}"))
                        .map(|value| value.to_string())
                })
                .collect::<Result<Vec<_>, _>>();
            if let Ok(colors) = colors {
                self.set(
                    "color-palette",
                    PreferenceValue::String(Some(colors.join(";"))),
                )?;
                migrated += 1;
            }
        }

        Ok(migrated)
    }
}

/// Returns the stable line-oriented contract emitted by the reference probe.
pub fn definition_contract() -> &'static str {
    DEFINITION_CONTRACT
}

/// Returns all preference definitions in the same order as the C class.
pub fn definitions() -> &'static [PreferenceDefinition] {
    static DEFINITIONS: OnceLock<Vec<PreferenceDefinition>> = OnceLock::new();
    DEFINITIONS.get_or_init(parse_definitions)
}

/// Finds a preference by its Xfconf property name.
pub fn definition(name: &str) -> Option<&'static PreferenceDefinition> {
    definitions()
        .iter()
        .find(|definition| definition.name == name)
}

fn parse_definitions() -> Vec<PreferenceDefinition> {
    DEFINITION_CONTRACT.lines().map(parse_definition).collect()
}

fn parse_definition(line: &'static str) -> PreferenceDefinition {
    let mut fields = line.split('\t');
    let name = fields.next().expect("preference name");
    let type_name = fields.next().expect("preference type");
    let default = fields.next().expect("preference default");
    let domain = fields.next().expect("preference domain");
    let legacy_key = fields.next().expect("legacy terminalrc key");
    assert!(fields.next().is_none(), "unexpected preference field");

    let kind = match type_name {
        "boolean" => PreferenceKind::Boolean,
        "string" => PreferenceKind::String,
        "uint" => {
            let (minimum, maximum) = parse_range(domain);
            PreferenceKind::Unsigned {
                minimum: minimum.parse().expect("unsigned minimum"),
                maximum: maximum.parse().expect("unsigned maximum"),
            }
        }
        "double" => {
            let (minimum, maximum) = parse_range(domain);
            PreferenceKind::Double {
                minimum: minimum.parse().expect("double minimum"),
                maximum: maximum.parse().expect("double maximum"),
            }
        }
        enumeration if enumeration.starts_with("enum:") => PreferenceKind::Enumeration {
            type_name: &enumeration["enum:".len()..],
            values: domain.split(',').collect(),
        },
        unsupported => panic!("unsupported preference type {unsupported}"),
    };

    PreferenceDefinition {
        name,
        kind,
        default: (default != "<null>").then_some(default),
        legacy_key,
    }
}

fn parse_range(range: &str) -> (&str, &str) {
    range.split_once(':').expect("bounded preference range")
}

fn required_definition(name: &str) -> Result<&'static PreferenceDefinition, PreferenceError> {
    definition(name).ok_or_else(|| PreferenceError(format!("unknown preference {name:?}")))
}

fn default_value(definition: &PreferenceDefinition) -> PreferenceValue {
    match &definition.kind {
        PreferenceKind::Boolean => {
            PreferenceValue::Boolean(definition.default.expect("boolean default") == "true")
        }
        PreferenceKind::String => PreferenceValue::String(definition.default.map(str::to_owned)),
        PreferenceKind::Unsigned { .. } => PreferenceValue::Unsigned(
            definition
                .default
                .expect("unsigned default")
                .parse()
                .expect("valid unsigned default"),
        ),
        PreferenceKind::Double { .. } => PreferenceValue::Double(
            definition
                .default
                .expect("double default")
                .parse()
                .expect("valid double default"),
        ),
        PreferenceKind::Enumeration { .. } => {
            PreferenceValue::Enumeration(definition.default.expect("enum default").to_owned())
        }
    }
}

fn validate(
    definition: &PreferenceDefinition,
    value: &PreferenceValue,
) -> Result<(), PreferenceError> {
    let valid = match (&definition.kind, value) {
        (PreferenceKind::Boolean, PreferenceValue::Boolean(_))
        | (PreferenceKind::String, PreferenceValue::String(_)) => true,
        (PreferenceKind::Unsigned { minimum, maximum }, PreferenceValue::Unsigned(value)) => {
            (*minimum..=*maximum).contains(value)
        }
        (PreferenceKind::Double { minimum, maximum }, PreferenceValue::Double(value)) => {
            value.is_finite() && (*minimum..=*maximum).contains(value)
        }
        (PreferenceKind::Enumeration { values, .. }, PreferenceValue::Enumeration(value)) => {
            values.contains(&value.as_str())
        }
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(PreferenceError(format!(
            "invalid value for preference {:?}",
            definition.name
        )))
    }
}

fn config_home() -> Result<PathBuf, PreferenceError> {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Ok(path);
        }
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
        .ok_or_else(|| PreferenceError("HOME is not set".to_owned()))
}

fn legacy_value(definition: &PreferenceDefinition, source: &str) -> PreferenceValue {
    match &definition.kind {
        PreferenceKind::Boolean => PreferenceValue::Boolean(source != "FALSE"),
        PreferenceKind::String => PreferenceValue::String(Some(source.to_owned())),
        PreferenceKind::Unsigned { .. } => PreferenceValue::Unsigned(libc::strtoul_u32(source)),
        PreferenceKind::Double { .. } => PreferenceValue::Double(libc::terminalrc_double(source)),
        PreferenceKind::Enumeration { values, .. } => {
            let value = values
                .iter()
                .copied()
                .find(|value| *value == source)
                .unwrap_or(values[0]);
            PreferenceValue::Enumeration(value.to_owned())
        }
    }
}
