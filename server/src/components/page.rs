use maud::{Markup, Render, html};

pub struct Page {
    pub title: String,
    pub content: Box<dyn Render>,
    pub flash: Option<String>,
}

impl Page {
    pub fn new(title: String, content: Box<dyn Render>, flash: Option<String>) -> Self {
        Self {
            title,
            content,
            flash,
        }
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

            body {
                @if let Some(flash_message) = &self.flash {
                    div class="flash-message" {
                        (flash_message)
                    }
                }

                (self.content.render())
            }
        }
    }
}

impl axum::response::IntoResponse for Page {
    fn into_response(self) -> axum::response::Response {
        self.render().into_response()
    }
}
