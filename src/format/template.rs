use std::path::{Path, PathBuf};
use crate::{Context, Error};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    header: String,
    footer: String,
}

impl Template {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        match Self::from_buffer(std::fs::read(path.as_ref())?) {
            Err(Error::Template { message, .. }) => {
                Err(Error::Template { file: PathBuf::from(path.as_ref()), message })
            },
            x => x,
        }
    }

    pub fn from_buffer(buf: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let ctx = Context::from_buffer(buf)?;
        let template = str::from_utf8(ctx.text())?;

        let markers = crate::query::Template::Match::from_node(ctx.root());
        let markers = if markers.len() == 1 {
            markers.into_iter().next().unwrap()
        } else {
            return Err(Error::Template {
                file: PathBuf::new(),
                message: "No template markers found".to_string(),
            });
        };

        // capture header trailing newline
        let mut header_end = markers.start.f.node.end_byte();
        let mut header = &template[..header_end];
        while !header.ends_with("\n") {
            header_end = template.ceil_char_boundary(header_end + 1);
            header = &template[..header_end];
        }

        // capture footer leading newline
        let mut footer_start = markers.end.f.node.start_byte();
        let mut footer = &template[footer_start..];
        while !footer.starts_with("\n") {
            footer_start = template.floor_char_boundary(footer_start - 1);
            footer = &template[footer_start..];
        }

        // capture footer leading CR too if present
        let one_more = template.floor_char_boundary(footer_start - 1);
        if template[one_more..].starts_with('\r') {
            footer = &template[one_more..];
        }

        Ok(Self {
            header: header.to_string(),
            footer: footer.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header() {
        let t = Template::from_buffer(crate::test::TEMPLATE_H_CTX.text())
            .expect("Failed to load template header");

        assert_eq!(t.header.len(), 237);
        assert!(t.header.ends_with('\n'));

        assert_eq!(t.footer.len(), 26);
        assert!(t.footer.starts_with('\r') || t.footer.starts_with('\n'));
    }
}