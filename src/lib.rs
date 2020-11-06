use anyhow::Result;
use font_kit::handle::Handle;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
    sync::Arc,
};

#[macro_export]
macro_rules! linked_list {
    ($($elem:expr),* $(,)?) => {{
        use std::collections::LinkedList;
        let mut l = LinkedList::new();
        $(l.push_back($elem);)*
        l
    }};
}

pub fn load_font_bytes(h: Handle) -> Result<Arc<Vec<u8>>> {
    match h {
        Handle::Memory { bytes, .. } => Ok(bytes),
        Handle::Path { path, .. } => Ok(Arc::new(
            BufReader::new(File::open(path)?)
                .bytes()
                .collect::<Result<Vec<_>, _>>()?,
        )),
    }
}
