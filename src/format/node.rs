use crate::{ContextCursor, ContextNode};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::{LazyLock, RwLock, RwLockWriteGuard},
};

const INDENT: &str = "  ";
static STRIP_MAP: LazyLock<HashMap<char, char>> =
    LazyLock::new(|| HashMap::from_iter([('\n', '␤'), ('\r', '␍')]));

fn sanitize(item: char) -> char {
    if let Some(clean) = STRIP_MAP.get(&item) {
        *clean
    } else {
        item
    }
}

static FMT_DISPLAY_CONFIG: RwLock<NodeDisplayConfig> =
    RwLock::new(NodeDisplayConfig::const_default());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeDisplayConfig {
    pub max_depth: usize,
    pub max_code: usize,
    pub show_anon: bool,
}

impl Default for NodeDisplayConfig {
    fn default() -> Self {
        Self::const_default()
    }
}

impl NodeDisplayConfig {
    const fn const_default() -> Self {
        Self {
            max_depth: 10,
            max_code: 50,
            show_anon: true,
        }
    }

    pub fn set(self) -> Result<(), std::sync::PoisonError<RwLockWriteGuard<'static, Self>>> {
        *(FMT_DISPLAY_CONFIG.write()?) = self;
        Ok(())
    }
}

impl Display for ContextNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = print_tree(f, self.walk())?;
        Ok(())
    }
}

pub fn print_tree<'t>(
    f: &mut std::fmt::Formatter,
    mut cursor: ContextCursor<'t>,
) -> Result<ContextCursor<'t>, std::fmt::Error> {
    let config = FMT_DISPLAY_CONFIG
        .read()
        .map_err(|_| std::fmt::Error)?
        .clone();

    if !config.show_anon && !cursor.cursor.node().is_named() {
        return Ok(cursor);
    }

    for _ in 0..cursor.cursor.depth() {
        write!(f, "{INDENT}")?;
    }

    writeln!(
        f,
        "{}",
        NodeFormat {
            node: cursor.node(),
            field: cursor.cursor.field_name(),
        },
    )?;

    if cursor.cursor.depth() as usize == config.max_depth - 1
        && cursor.node().node.named_child_count() > 0
    {
        for _ in 0..config.max_depth {
            write!(f, "{INDENT}")?;
        }
        writeln!(f, "+{} children", cursor.node().node.named_child_count())?;
    } else if cursor.cursor.goto_first_child() {
        cursor = print_tree(f, cursor)?;
        while cursor.cursor.goto_next_sibling() {
            cursor = print_tree(f, cursor)?;
        }
        cursor.cursor.goto_parent();
    }

    Ok(cursor)
}

pub struct NodeFormat<'a> {
    pub node: ContextNode<'a>,
    pub field: Option<&'static str>,
}

impl Display for NodeFormat<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = FMT_DISPLAY_CONFIG
            .read()
            .map_err(|_| std::fmt::Error)?
            .clone();

        let body = self.node.as_str();
        let rest = body.chars().skip(config.max_code).count();
        let body: String = body.chars().take(config.max_code).map(sanitize).collect();

        let body = if rest > 0 {
            format!("\"{body}...\" +{rest} chars")
        } else {
            format!("\"{body}\"")
        };

        match self.field {
            Some(field) if self.node.node.is_named() => {
                write!(f, "({field}) {typ} {body}", typ = self.node.node.kind())
            }
            Some(field) => {
                write!(f, "({field}) -{typ} {body}", typ = self.node.node.kind())
            }
            None if self.node.node.is_named() => {
                write!(f, "{typ} {body}", typ = self.node.node.kind())
            }
            None => {
                write!(f, "-{typ} {body}", typ = self.node.node.kind())
            }
        }
    }
}
