use crate::config::Config;
use crate::parse::Blox;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub struct NumberMap(HashMap<String, usize>);

impl Deref for NumberMap {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NumberMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl NumberMap {
    pub fn new(config: &Config) -> Self {
        Self(
            config
                .environments
                .iter()
                .map(|(env, _)| (env.clone(), 1))
                .collect(),
        )
    }
    pub fn reset(&mut self, config: &Config) {
        self.iter_mut()
            .filter(|(k, _)| config.prefix_number(k))
            .for_each(|(_, v)| *v = 1);
    }
    pub fn set_blox(&mut self, blox: &mut Blox, section_number: Option<&str>) -> Result<()> {
        let n = self
            .get_mut(blox.env())
            .context("Couldn't find environment")?;

        if blox.set_number(*n, section_number) {
            *n += 1;
        }

        Ok(())
    }
}
