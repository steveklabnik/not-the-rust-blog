use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

use handlebars::Handlebars;

struct Blog {
    handlebars: Handlebars,
    posts: Vec<Post>,
    out_directory: PathBuf,
}

#[derive(Debug)]
struct Post {
    filename: String,
    title: String,
    author: String,
    year: String,
    month: String,
    day: String,
}

impl Blog {
    fn new<T>(out_directory: T, posts_directory: T) -> Result<Blog, Box<Error>>
    where
        T: Into<PathBuf>,
    {
        let mut handlebars = Handlebars::new();

        handlebars.set_strict_mode(true);

        handlebars.register_templates_directory(".hbs", "templates")?;

        let posts = Blog::load_posts(posts_directory.into())?;

        Ok(Blog {
            handlebars,
            posts,
            out_directory: out_directory.into(),
        })
    }

    fn load_posts(dir: PathBuf) -> Result<Vec<Post>, Box<Error>> {
        let mut posts = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // yeah this might blow up, but it won't
            let filename = path.file_name().unwrap().to_str().unwrap();

            let mut split = filename.splitn(4, "-");

            let year = split.next().unwrap().to_string();
            let month = split.next().unwrap().to_string();
            let day = split.next().unwrap().to_string();
            let filename = split.next().unwrap().to_string();

            let author = String::from("steve");
            let title = String::from(&*filename);

            let post = Post {
                filename,
                title,
                author,
                year,
                month,
                day,
            };

            println!("Found post: {:?}", post);
        }

        Ok(posts)
    }

    fn render(&self) -> Result<(), Box<Error>> {
        fs::create_dir_all(&self.out_directory)?;

        self.render_index()?;

        Ok(())
    }

    fn render_index(&self) -> Result<(), Box<Error>> {
        let mut data = BTreeMap::new();
        data.insert("title".to_string(), "The Rust Programming Language Blog".to_string());

        self.render_template("index.md", "index", data)?;
        
        Ok(())
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
    let blog = Blog::new("site", "posts")?;

    blog.render()?;

    Ok(())
}
