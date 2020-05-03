// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod battery;
mod brightness;
mod cpu;
mod date_time;
mod error;
mod memory;
mod module;
mod nl_data;
mod pulse;
mod temperature;
mod wireless;
use battery::Battery;
use brightness::Brightness;
use cpu::Cpu;
use date_time::DateTime as MDateTime;
use error::Error;
use memory::Memory;
use module::Module;
use pulse::{Pulse, PulseData};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;
use temperature::Temperature;
use wireless::Wireless;

const DEFAULT_FONT: &'static str = "+@fn=0;";
const ICON_FONT: &'static str = "+@fn=1;";
const DEFAULT_COLOR: &'static str = "+@fg=0;";
const RED: &'static str = "+@fg=1;";
const GREEN: &'static str = "+@fg=2;";
const PULSE_RATE: Duration = Duration::from_millis(16);
const SINK_INDEX: u32 = 0;
const SOURCE_INDEX: u32 = 1;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ModuleConfig {
    DateTime,
    Battery,
    Brightness,
    Cpu,
    Temperature,
    Sound,
    Mic,
    Wireless,
    Memory,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub tick: Option<u32>,
    default_font: String,
    icon_font: String,
    default_color: String,
    red: String,
    green: String,
    sink: u32,
    source: u32,
    modules: Vec<ModuleConfig>,
    cpu_tick: Option<u32>,
    wireless_tick: Option<u32>,
    pulse_tick: Option<u32>,
    proc_stat: Option<String>,
    proc_meminfo: Option<String>,
    energy_now: Option<String>,
    power_status: Option<String>,
    energy_full_design: Option<String>,
    coretemp: Option<String>,
    backlight: Option<String>,
}

trait Refresh {
    fn refresh(&mut self) -> Result<String, Error>;
}

pub struct Bar<'a> {
    default_font: &'a str,
    icon_font: &'a str,
    default_color: &'a str,
    red: &'a str,
    green: &'a str,
    pulse: Pulse,
    prev_sink: Option<PulseData>,
    prev_source: Option<PulseData>,
    config: &'a Config,
    modules: Vec<Module<'a>>,
}

impl<'a> Bar<'a> {
    pub fn with_config(config: &'a Config) -> Result<Self, Error> {
        let mut modules = vec![];
        for module in &config.modules {
            match module {
                ModuleConfig::DateTime => modules.push(Module::DateTime(MDateTime::new())),
                ModuleConfig::Battery => {
                    modules.push(Module::Battery(Battery::with_config(&config)))
                }
                ModuleConfig::Memory => modules.push(Module::Memory(Memory::with_config(&config))),
                ModuleConfig::Brightness => {
                    modules.push(Module::Brightness(Brightness::with_config(&config)))
                }
                ModuleConfig::Temperature => {
                    modules.push(Module::Temperature(Temperature::with_config(&config)?))
                }
                ModuleConfig::Cpu => modules.push(Module::Cpu(Cpu::with_config(&config))),
                ModuleConfig::Wireless => {
                    modules.push(Module::Wireless(Wireless::with_config(&config)))
                }
                _ => return Err(Error::new("unknown module")),
            }
        }
        Ok(Bar {
            default_font: DEFAULT_FONT,
            icon_font: ICON_FONT,
            default_color: DEFAULT_COLOR,
            red: RED,
            green: GREEN,
            pulse: Pulse::new(PULSE_RATE, SINK_INDEX, SOURCE_INDEX),
            prev_sink: None,
            prev_source: None,
            config,
            modules,
        })
    }

    fn sound(&mut self) -> Result<String, Error> {
        let data = self.pulse.output_data();
        if data.is_some() {
            self.prev_sink = data;
        }
        let icon;
        let mut color = self.default_color;
        if let Some(info) = self.prev_sink {
            if info.1 {
                icon = "󰸈";
            } else {
                icon = match info.0 {
                    0..=9 => "󰕿",
                    10..=40 => "󰖀",
                    _ => "󰕾",
                }
            }
            if info.0 > 150 {
                color = self.red;
            }
            Ok(format!(
                "{:3}% {}{}{}{}{}",
                info.0, color, self.icon_font, icon, self.default_font, self.default_color
            ))
        } else {
            icon = "󰖁";
            Ok(format!(
                "     {}{}{}",
                self.icon_font, icon, self.default_font
            ))
        }
    }

    fn mic(&mut self) -> Result<String, Error> {
        let data = self.pulse.input_data();
        if data.is_some() {
            self.prev_source = data;
        }
        let icon;
        let mut color = self.default_color;
        if let Some(info) = self.prev_source {
            if info.1 {
                icon = "󰍭";
            } else {
                icon = "󰍬";
            }
            if info.0 > 150 {
                color = self.red;
            }
            Ok(format!(
                "{:3}% {}{}{}{}{}",
                info.0, color, self.icon_font, icon, self.default_font, self.default_color
            ))
        } else {
            icon = "󰍮";
            Ok(format!(
                "     {}{}{}",
                self.icon_font, icon, self.default_font
            ))
        }
    }

    pub fn update(&mut self) -> Result<(), Error> {
        // println!(
        // "{}  {}  {}  {}  {}  {}  {}  {}   {}",
        // memory, cpu, temperature, brightness, mic, sound, wireless, battery, date_time
        // );
        for module in &mut self.modules {
            println!("{}", module.refresh()?);
        }
        println!("");
        Ok(())
    }
}

fn read_and_trim<'a>(file: &'a str) -> Result<String, Error> {
    let content = fs::read_to_string(file)
        .map_err(|err| format!("error while reading the file \"{}\": {}", file, err))?;
    Ok(content.trim().to_string())
}

fn read_and_parse<'a>(file: &'a str) -> Result<i32, Error> {
    let content = read_and_trim(file)?;
    let data = content
        .parse::<i32>()
        .map_err(|err| format!("error while parsing the file \"{}\": {}", file, err))?;
    Ok(data)
}
