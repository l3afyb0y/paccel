use std::{
    fmt::{Debug, Display},
    fs::{File, OpenOptions},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::{
    fixedptc::Fpt,
    params::{
        AccelMode, CommonParamArgs, LinearParamArgs, NaturalParamArgs, Param, SynchronousParamArgs,
        format_param_value, validate_param_value,
    },
};

const DEVICE_PATH: &str = "/dev/paccel";
const ABI_VERSION: u16 = 1;
const CONFIG_MODE: u32 = 0o600;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaccelConfigV1 {
    abi_version: u16,
    struct_size: u16,
    mode: u8,
    reserved: [u8; 3],
    sens_mult: Fpt,
    yx_ratio: Fpt,
    input_dpi: Fpt,
    angle_rotation: Fpt,
    accel: Fpt,
    offset: Fpt,
    output_cap: Fpt,
    decay_rate: Fpt,
    limit: Fpt,
    gamma: Fpt,
    smooth: Fpt,
    motivity: Fpt,
    sync_speed: Fpt,
}

const fn ioctl_code(direction: u64, number: u64) -> libc::c_ulong {
    ((direction << 30)
        | ((size_of::<PaccelConfigV1>() as u64) << 16)
        | ((b'p' as u64) << 8)
        | number) as libc::c_ulong
}
const GET_CONFIG: libc::c_ulong = ioctl_code(2, 1);
const SET_CONFIG: libc::c_ulong = ioctl_code(1, 2);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UserConfig {
    pub mode: AccelMode,
    pub sens_mult: f64,
    pub yx_ratio: f64,
    pub input_dpi: f64,
    pub angle_rotation: f64,
    pub accel: f64,
    pub offset: f64,
    pub output_cap: f64,
    pub decay_rate: f64,
    pub limit: f64,
    pub gamma: f64,
    pub smooth: f64,
    pub motivity: f64,
    pub sync_speed: f64,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            mode: AccelMode::Linear,
            sens_mult: 1.0,
            yx_ratio: 1.0,
            input_dpi: 1000.0,
            angle_rotation: 0.0,
            accel: 0.0,
            offset: 0.0,
            output_cap: 0.0,
            decay_rate: 0.1,
            limit: 1.5,
            gamma: 1.0,
            smooth: 0.5,
            motivity: 1.5,
            sync_speed: 5.0,
        }
    }
}

impl TryFrom<&UserConfig> for PaccelConfigV1 {
    type Error = anyhow::Error;
    fn try_from(value: &UserConfig) -> Result<Self, Self::Error> {
        if !value.sens_mult.is_finite()
            || value.sens_mult <= 0.0
            || !value.yx_ratio.is_finite()
            || value.yx_ratio <= 0.0
            || !value.input_dpi.is_finite()
            || value.input_dpi <= 0.0
        {
            bail!("sensitivity, axis ratio, and input DPI must be finite and positive");
        }
        for (param, number) in [
            (Param::Accel, value.accel),
            (Param::OffsetLinear, value.offset),
            (Param::OutputCap, value.output_cap),
            (Param::DecayRate, value.decay_rate),
            (Param::Limit, value.limit),
            (Param::Gamma, value.gamma),
            (Param::Smooth, value.smooth),
            (Param::Motivity, value.motivity),
            (Param::SyncSpeed, value.sync_speed),
        ] {
            validate_param_value(param, number)?;
        }
        Ok(Self {
            abi_version: ABI_VERSION,
            struct_size: size_of::<Self>() as u16,
            mode: value.mode as u8,
            reserved: [0; 3],
            sens_mult: value.sens_mult.into(),
            yx_ratio: value.yx_ratio.into(),
            input_dpi: value.input_dpi.into(),
            angle_rotation: value.angle_rotation.into(),
            accel: value.accel.into(),
            offset: value.offset.into(),
            output_cap: value.output_cap.into(),
            decay_rate: value.decay_rate.into(),
            limit: value.limit.into(),
            gamma: value.gamma.into(),
            smooth: value.smooth.into(),
            motivity: value.motivity.into(),
            sync_speed: value.sync_speed.into(),
        })
    }
}

impl From<PaccelConfigV1> for UserConfig {
    fn from(v: PaccelConfigV1) -> Self {
        Self {
            mode: match v.mode {
                0 => AccelMode::Linear,
                1 => AccelMode::Natural,
                2 => AccelMode::Synchronous,
                _ => AccelMode::NoAccel,
            },
            sens_mult: v.sens_mult.into(),
            yx_ratio: v.yx_ratio.into(),
            input_dpi: v.input_dpi.into(),
            angle_rotation: v.angle_rotation.into(),
            accel: v.accel.into(),
            offset: v.offset.into(),
            output_cap: v.output_cap.into(),
            decay_rate: v.decay_rate.into(),
            limit: v.limit.into(),
            gamma: v.gamma.into(),
            smooth: v.smooth.into(),
            motivity: v.motivity.into(),
            sync_speed: v.sync_speed.into(),
        }
    }
}

pub trait ParamStore: Debug {
    fn set(&mut self, param: Param, value: f64) -> anyhow::Result<()>;
    fn get(&self, param: Param) -> anyhow::Result<Fpt>;
    fn set_current_accel_mode(&mut self, mode: AccelMode) -> anyhow::Result<()>;
    fn get_current_accel_mode(&self) -> anyhow::Result<AccelMode>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DeviceStore;

impl DeviceStore {
    pub fn read_config(&self) -> anyhow::Result<UserConfig> {
        let file = File::open(DEVICE_PATH)
            .context("open /dev/paccel (is the driver loaded and are you in the paccel group?)")?;
        let mut raw = PaccelConfigV1::try_from(&UserConfig::default())?;
        // SAFETY: the ioctl ABI accepts a writable pointer to this exact repr(C) structure.
        if unsafe { libc::ioctl(file.as_raw_fd(), GET_CONFIG, &mut raw) } < 0 {
            return Err(std::io::Error::last_os_error()).context("read paccel configuration");
        }
        if raw.abi_version != ABI_VERSION || raw.struct_size as usize != size_of::<PaccelConfigV1>()
        {
            bail!("unsupported paccel driver configuration ABI");
        }
        Ok(raw.into())
    }

    pub fn write_config(&self, config: &UserConfig) -> anyhow::Result<()> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(DEVICE_PATH)
            .context("open /dev/paccel for writing (check group membership)")?;
        let raw = PaccelConfigV1::try_from(config)?;
        // SAFETY: the ioctl reads a pointer to this initialized repr(C) structure.
        if unsafe { libc::ioctl(file.as_raw_fd(), SET_CONFIG, &raw) } < 0 {
            return Err(std::io::Error::last_os_error()).context("apply paccel configuration");
        }
        Ok(())
    }

    fn update(&self, change: impl FnOnce(&mut UserConfig)) -> anyhow::Result<()> {
        let mut config = self.read_config()?;
        change(&mut config);
        self.write_config(&config)
    }
    pub fn set_all_common(&mut self, a: CommonParamArgs) -> anyhow::Result<()> {
        self.update(|c| {
            c.sens_mult = a.sens_mult;
            c.yx_ratio = a.yx_ratio;
            c.input_dpi = a.input_dpi;
            c.angle_rotation = a.angle_rotation;
        })
    }
    pub fn set_all_linear(&mut self, a: LinearParamArgs) -> anyhow::Result<()> {
        self.update(|c| {
            c.accel = a.accel;
            c.offset = a.offset_linear;
            c.output_cap = a.output_cap;
        })
    }
    pub fn set_all_natural(&mut self, a: NaturalParamArgs) -> anyhow::Result<()> {
        self.update(|c| {
            c.decay_rate = a.decay_rate;
            c.offset = a.offset_natural;
            c.limit = a.limit;
        })
    }
    pub fn set_all_synchronous(&mut self, a: SynchronousParamArgs) -> anyhow::Result<()> {
        self.update(|c| {
            c.gamma = a.gamma;
            c.smooth = a.smooth;
            c.motivity = a.motivity;
            c.sync_speed = a.sync_speed;
        })
    }
}

impl ParamStore for DeviceStore {
    fn set(&mut self, p: Param, value: f64) -> anyhow::Result<()> {
        validate_param_value(p, value)?;
        self.update(|c| match p {
            Param::SensMult => c.sens_mult = value,
            Param::YxRatio => c.yx_ratio = value,
            Param::InputDpi => c.input_dpi = value,
            Param::AngleRotation => c.angle_rotation = value,
            Param::Accel => c.accel = value,
            Param::OffsetLinear | Param::OffsetNatural => c.offset = value,
            Param::OutputCap => c.output_cap = value,
            Param::DecayRate => c.decay_rate = value,
            Param::Limit => c.limit = value,
            Param::Gamma => c.gamma = value,
            Param::Smooth => c.smooth = value,
            Param::Motivity => c.motivity = value,
            Param::SyncSpeed => c.sync_speed = value,
        })
    }
    fn get(&self, p: Param) -> anyhow::Result<Fpt> {
        let c = PaccelConfigV1::try_from(&self.read_config()?)?;
        Ok(match p {
            Param::SensMult => c.sens_mult,
            Param::YxRatio => c.yx_ratio,
            Param::InputDpi => c.input_dpi,
            Param::AngleRotation => c.angle_rotation,
            Param::Accel => c.accel,
            Param::OffsetLinear | Param::OffsetNatural => c.offset,
            Param::OutputCap => c.output_cap,
            Param::DecayRate => c.decay_rate,
            Param::Limit => c.limit,
            Param::Gamma => c.gamma,
            Param::Smooth => c.smooth,
            Param::Motivity => c.motivity,
            Param::SyncSpeed => c.sync_speed,
        })
    }
    fn set_current_accel_mode(&mut self, mode: AccelMode) -> anyhow::Result<()> {
        self.update(|c| c.mode = mode)
    }
    fn get_current_accel_mode(&self) -> anyhow::Result<AccelMode> {
        Ok(self.read_config()?.mode)
    }
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(dir).join("paccel/config.toml"));
    }
    let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home).join(".config/paccel/config.toml"))
}

pub fn save_config(config: &UserConfig, path: &Path, force: bool) -> anyhow::Result<()> {
    let _ = PaccelConfigV1::try_from(config)?;
    if path.exists() && !force {
        bail!(
            "{} already exists; use --force to replace it",
            path.display()
        );
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("configuration path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let temp = path.with_extension(format!("toml.tmp-{}", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(CONFIG_MODE);
    let mut file = options
        .open(&temp)
        .with_context(|| format!("create {}", temp.display()))?;
    use std::io::Write;
    file.write_all(toml::to_string_pretty(config)?.as_bytes())?;
    file.sync_all()?;
    std::fs::rename(&temp, path).with_context(|| format!("replace {}", path.display()))?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

pub fn load_config(path: &Path) -> anyhow::Result<UserConfig> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let config: UserConfig =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let _ = PaccelConfigV1::try_from(&config)?;
    Ok(config)
}

/// Import the currently active legacy maccel values without modifying that driver.
pub fn import_maccel_config() -> anyhow::Result<UserConfig> {
    let root = Path::new("/sys/module/maccel/parameters");
    let mut config = UserConfig::default();
    let read_fixed = |name: &str| -> anyhow::Result<f64> {
        let text = std::fs::read_to_string(root.join(name))
            .with_context(|| format!("read legacy maccel parameter {name}"))?;
        let raw: i64 = text
            .trim()
            .parse()
            .with_context(|| format!("parse legacy maccel parameter {name}"))?;
        Ok(Fpt(raw).into())
    };
    let mode: u8 = std::fs::read_to_string(root.join("MODE"))
        .context("read legacy maccel mode")?
        .trim()
        .parse()?;
    config.mode = match mode {
        0 => AccelMode::Linear,
        1 => AccelMode::Natural,
        2 => AccelMode::Synchronous,
        3 => AccelMode::NoAccel,
        _ => bail!("legacy maccel has an unknown mode {mode}"),
    };
    config.sens_mult = read_fixed("SENS_MULT")?;
    config.yx_ratio = read_fixed("YX_RATIO")?;
    config.input_dpi = read_fixed("INPUT_DPI")?;
    config.angle_rotation = read_fixed("ANGLE_ROTATION")?;
    config.accel = read_fixed("ACCEL")?;
    config.offset = read_fixed("OFFSET")?;
    config.output_cap = read_fixed("OUTPUT_CAP")?;
    config.decay_rate = read_fixed("DECAY_RATE")?;
    config.limit = read_fixed("LIMIT")?;
    config.gamma = read_fixed("GAMMA")?;
    config.smooth = read_fixed("SMOOTH")?;
    config.motivity = read_fixed("MOTIVITY")?;
    config.sync_speed = read_fixed("SYNC_SPEED")?;
    let _ = PaccelConfigV1::try_from(&config)?;
    Ok(config)
}

impl Display for Fpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format_param_value(f64::from(*self)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_matches_kernel_abi() {
        assert_eq!(size_of::<PaccelConfigV1>(), 112);
    }
    #[test]
    fn user_config_round_trips_through_driver_layout() {
        let source = UserConfig {
            mode: AccelMode::Synchronous,
            sens_mult: 0.75,
            yx_ratio: 1.0,
            input_dpi: 26000.0,
            angle_rotation: 3.0,
            accel: 0.25,
            offset: 1.5,
            output_cap: 2.0,
            decay_rate: 0.125,
            limit: 1.5,
            gamma: 1.0,
            smooth: 0.5,
            motivity: 2.0,
            sync_speed: 5.0,
        };
        let driver = PaccelConfigV1::try_from(&source).unwrap();
        assert_eq!(UserConfig::from(driver), source);
    }
    #[test]
    fn invalid_user_config_is_rejected_before_ioctl() {
        let c = UserConfig {
            input_dpi: 0.,
            ..UserConfig::default()
        };
        assert!(PaccelConfigV1::try_from(&c).is_err());
    }
}
