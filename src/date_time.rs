// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::Error;
use crate::module::BaruMod;
use crate::Config as MainConfig;
use crate::Pulse;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const PLACEHOLDER: &str = "+@fn=1;󰸗+@fn=0;";
const FORMAT: &str = "%a. %-e %B %Y, %-kh%M";
const TICK_RATE: Duration = Duration::from_millis(500);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    format: Option<String>,
    tick: Option<u32>,
    placeholder: Option<String>,
}

#[derive(Debug)]
pub struct InternalConfig<'a> {
    format: &'a str,
    tick: Duration,
}

impl<'a> From<&'a MainConfig> for InternalConfig<'a> {
    fn from(config: &'a MainConfig) -> Self {
        let mut tick = TICK_RATE;
        let mut format = FORMAT;
        if let Some(c) = &config.date_time {
            if let Some(d) = &c.format {
                format = d;
            }
            if let Some(t) = c.tick {
                tick = Duration::from_millis(t as u64)
            }
        }
        InternalConfig { format, tick }
    }
}

#[derive(Debug)]
pub struct DateTime<'a> {
    placeholder: &'a str,
    config: &'a MainConfig,
}

impl<'a> DateTime<'a> {
    pub fn with_config(config: &'a MainConfig) -> Self {
        let mut placeholder = PLACEHOLDER;
        if let Some(c) = &config.date_time {
            if let Some(p) = &c.placeholder {
                placeholder = p
            }
        }
        DateTime {
            placeholder,
            config,
        }
    }
}

impl<'a> BaruMod for DateTime<'a> {
    fn run_fn(&self) -> fn(MainConfig, Arc<Mutex<Pulse>>, Sender<String>) -> Result<(), Error> {
        run
    }

    fn placeholder(&self) -> &str {
        self.placeholder
    }
}

pub fn run(main_config: MainConfig, _: Arc<Mutex<Pulse>>, tx: Sender<String>) -> Result<(), Error> {
    let config = InternalConfig::from(&main_config);
    loop {
        let now = Local::now();
        tx.send(now.format(config.format).to_string())?;
        thread::sleep(config.tick);
    }
}
