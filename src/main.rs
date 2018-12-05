use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

use handlebars::Handlebars;

use serde_derive::{Deserialize, Serialize};

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
    contents: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct YamlHeader {
    title: String,
    author: String,
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

            // we need to get the metadata out of the url
            let mut split = filename.splitn(4, "-");

            let year = split.next().unwrap().to_string();
            let month = split.next().unwrap().to_string();
            let day = split.next().unwrap().to_string();
            let filename = split.next().unwrap().to_string();

            // now we need to get the data from the post itself
            let author = String::from("steve");
            let title = String::from(&*filename);

            let contents = fs::read_to_string(path)?;

            // yaml headers.... we know the first four bytes of each file are "---\n"
            // so we need to find the end. we need the fours to adjust for those first bytes
            let end_of_yaml = contents[4..].find("---").unwrap() + 4;
            let yaml = &contents[..end_of_yaml];

            let YamlHeader { author, title } = serde_yaml::from_str(yaml)?;

            // finally, the contents. we add + to get rid of the final "---\n\n"
            let contents = contents[end_of_yaml + 5..].to_string();

            let post = Post {
                filename,
                title,
                author,
                year,
                month,
                day,
                contents,
            };

            posts.push(post);
        }

        // finally, sort the posts. oldest first.

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
        data.insert("parent".to_string(), "layout".to_string());

        self.render_template("index.html", "index", data)?;
        
        Ok(())
    }

    fn render_template(&self, name: &str, template: &str, data: BTreeMap<String, String>) -> Result<(), Box<Error>> {
        let out_file = self.out_directory.join(name);

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
