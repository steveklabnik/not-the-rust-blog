use std::{
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

use handlebars::Handlebars;

use serde_derive::{Deserialize, Serialize};
use serde_json::json;

struct Blog {
    handlebars: Handlebars,
    posts: Vec<Post>,
    out_directory: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    filename: String,
    title: String,
    author: String,
    year: String,
    month: String,
    day: String,
    contents: String,
    url: String,
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

            let contents = fs::read_to_string(path)?;

            // yaml headers.... we know the first four bytes of each file are "---\n"
            // so we need to find the end. we need the fours to adjust for those first bytes
            let end_of_yaml = contents[4..].find("---").unwrap() + 4;
            let yaml = &contents[..end_of_yaml];

            let YamlHeader { author, title } = serde_yaml::from_str(yaml)?;

            // next, the contents. we add + to get rid of the final "---\n\n"
            let contents = contents[end_of_yaml + 5..].to_string();

            // finally, the url.
            let mut url = PathBuf::from(&*filename);
            url.set_extension("html");

            // this is fine
            let url = format!("{}/{}/{}/{}", year, month, day, url.to_str().unwrap());

            let post = Post {
                filename,
                title,
                author,
                year,
                month,
                day,
                contents,
                url,
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
        /*
        data.insert("title".to_string(), "The Rust Programming Language Blog".to_string());
        data.insert("parent".to_string(), "layout".to_string());

        data.insert("post_titles".to_string(), self.posts)
        */

        let data = json!({
            "title": "The Rust Programming Language Blog",
            "parent": "layout",
            "posts": self.posts,
        });

        self.render_template("index.html", "index", data)?;
        
        Ok(())
    }

    fn render_template(&self, name: &str, template: &str, data: serde_json::Value) -> Result<(), Box<Error>> {
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
