use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

use handlebars::Handlebars;

struct Config {
    handlebars: Handlebars,
    out_directory: PathBuf,
}

impl Config {
    fn new<T>(out_directory: T) -> Result<Config, Box<Error>>
    where
        T: Into<PathBuf>,
    {
        let mut handlebars = Handlebars::new();

        handlebars.set_strict_mode(true);

        handlebars.register_templates_directory(".hbs", "templates")?;

        Ok(Config {
            handlebars,
            out_directory: out_directory.into(),
        })
    }

    fn create_out_directory(&self) -> Result<(), Box<Error>> {
        Ok(fs::create_dir_all(&self.out_directory)?)
    }

    fn render_template(&self, name: &str, template: &str, data: BTreeMap<String, String>) -> Result<(), Box<Error>> {
        let mut html = PathBuf::from(name);
        html.set_extension("html");

        let out_file = self.out_directory.join(html);

        let file = File::create(out_file)?;

        self.handlebars.render_to_write(template, &data, file)?;

        Ok(())
    }
}

fn main() -> Result<(), Box<Error>> {
    let config = Config::new("site")?;

    config.create_out_directory()?;

    let mut data = BTreeMap::new();
    data.insert("world".to_string(), "世界!".to_string());

    config.render_template("index.md", "index", data)?;

    Ok(())
}
