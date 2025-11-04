use crate::config::processor_config::Config;

pub trait Render {
    fn html(&self, config: &Config) -> String;
    fn latex(&self, config: &Config) -> String {
        self.html(config)
    }
}
