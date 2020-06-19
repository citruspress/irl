use rppal::pwm;
use rppal::pwm::{Channel, Polarity, Pwm};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::io;
use std::time::Duration;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("PWM error"))]
    InitPwm { source: pwm::Error },
    #[snafu(display("Unable to read config from {}", path))]
    ReadConfig { source: io::Error, path: String },
    #[snafu(display("Unable to read config from {}", path))]
    ParseConfig {
        source: toml::de::Error,
        path: String,
    },
    #[snafu(display("Signal not found {}", signal))]
    SignalNotFound { signal: String },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Remote {
    pwm: Pwm,
    config: RemoteConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoteConfig {
    pub bits: u8,
    pub header: Bits,
    pub one: Bits,
    pub zero: Bits,
    pub gap: Bits,
    pub repeat: u8,
    pub frequency: f64,
    pub address: u32,
    pub codes: Vec<Code>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Bits(u32, u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct Code {
    pub signal: String,
    pub code: u32,
}

impl Remote {
    pub fn from_config(path: &str) -> Result<Remote> {
        let text = std::fs::read_to_string(path).context(ReadConfig { path })?;
        let config: RemoteConfig = toml::from_str(&text).context(ParseConfig { path })?;

        let pwm = Pwm::with_frequency(
            Channel::Pwm0,
            config.frequency,
            0.5f64,
            Polarity::Normal,
            false,
        )
        .context(InitPwm {})?;

        Ok(Remote {
            pwm: pwm,
            config: config,
        })
    }

    pub fn emit(&self, signal_name: &str) -> Result<()> {
        for code in &self.config.codes {
            if code.signal == signal_name {
                self.emit_code(code.code)?;

                return Ok(());
            }
        }

        Err(Error::SignalNotFound {
            signal: signal_name.to_string(),
        })
    }

    fn emit_code(&self, code: u32) -> Result<()> {
        for _ in 0..self.config.repeat {
            self.emit_bit(self.config.header)?;
            self.emit_data(self.config.address)?;
            self.emit_data(code)?;
            self.emit_bit(self.config.gap)?;
        }
        Ok(())
    }

    fn emit_data(&self, data: u32) -> Result<()> {
        let bits = self.config.bits;
        for n in 0..bits {
            let bit = (data >> ((bits - n) - 1)) & 1;

            if bit == 0 {
                self.emit_bit(self.config.zero)?;
            } else {
                self.emit_bit(self.config.one)?;
            }
        }

        Ok(())
    }

    fn emit_bit(&self, bit: Bits) -> Result<()> {
        self.pwm.enable().context(InitPwm {})?;
        spin_sleep::sleep(Duration::from_micros(bit.0.into()));
        self.pwm.disable().context(InitPwm {})?;
        spin_sleep::sleep(Duration::from_micros(bit.1.into()));

        Ok(())
    }
}
