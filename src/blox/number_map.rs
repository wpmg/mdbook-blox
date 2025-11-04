use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::config::processor_config::Config;

#[derive(Debug, Default)]
pub struct BloxNumber(usize, bool);

impl BloxNumber {
    #[inline]
    pub fn new(prefix: bool) -> Self {
        Self(0, prefix)
    }
    #[inline]
    pub fn get(&self) -> usize {
        self.0
    }
    #[inline]
    pub fn reset(&mut self) {
        self.0 = 0;
    }
    #[inline]
    pub fn get_number_increment(&mut self) -> usize {
        self.0 += 1;
        self.0
    }
    pub fn next_string(&mut self, section_number: Option<&str>) -> String {
        let number = self.get_number_increment();

        if self.1
            && let Some(prefix) = section_number
        {
            format!("{prefix}{number}")
        } else {
            number.to_string()
        }
    }
}

#[derive(Debug)]
pub struct NumberMap {
    map: HashMap<String, BloxNumber>,
    figure: BloxNumber,
    section_number: Option<String>,
}

impl NumberMap {
    pub fn new(config: &Config) -> Self {
        Self {
            map: config
                .get_environment_keys()
                .map(|env| (env.clone(), BloxNumber::new(config.env_prefix_number(env))))
                .collect(),
            figure: BloxNumber::new(config.fig_prefix_number()),
            section_number: None,
        }
    }
    pub fn reset(&mut self, section_number: Option<String>) {
        self.section_number = section_number;
        self.map.iter_mut().for_each(|(_, v)| v.reset());
        self.figure.reset();
    }
    pub fn next_string(&mut self, env: &str) -> Result<String> {
        let bn = self.map.get_mut(env).context("Couldn't find environment")?;
        Ok(bn.next_string(self.section_number.as_deref()))
    }
    pub fn next_figure_string(&mut self) -> String {
        self.figure.next_string(self.section_number.as_deref())
    }
}
