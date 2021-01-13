use actix_web::body::Body;
use std::io::Write;

#[macro_export]
macro_rules! render {
    ($template:path) => (super::actix_ructe::Render(|o| $template(o)));
    ($template:path, $($arg:expr),*) => {{
        use actix_ructe::Render;
        Render(|o| $template(o, $($arg),*))
    }};
    ($template:path, $($arg:expr),* ,) => {{
        use actix_ructe::Render;
        Render(|o| $template(o, $($arg),*))
    }};
}

pub struct Render<T: FnOnce(&mut dyn Write) -> std::io::Result<()>>(pub T);

impl<T: FnOnce(&mut dyn Write) -> std::io::Result<()>> From<Render<T>> for Body {
    fn from(t: Render<T>) -> Self {
        let mut buf = Vec::new();
        match t.0(&mut buf) {
            Ok(()) => buf.into(),
            Err(_e) => {
                //log::warn!("Failed to render: {}", e);
                "Render failed".into()
            }
        }
    }
}
