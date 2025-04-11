use maud::{Markup, Render, html};

pub struct Page {
    pub title: String,
    pub content: Box<dyn Render>,
}

impl Page {
    pub fn new(title: String, content: Box<dyn Render>) -> Self {
        Self { title, content }
    }
}

impl Render for Page {
    fn render(&self) -> Markup {
        html! {
            head {
                title { (self.title) }
                link rel="stylesheet" href="/static/styles.css";
                script src="/static/viewTransition.js" {}
            }

            (self.content.render())
        }
    }
}

impl axum::response::IntoResponse for Page {
    fn into_response(self) -> axum::response::Response {
        self.render().into_response()
    }
}
