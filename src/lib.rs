#[macro_export]
macro_rules! linked_list {
    ($($elem:expr),* $(,)?) => {{
        use std::collections::LinkedList;
        let mut l = LinkedList::new();
        $(l.push_back($elem);)*
        l
    }};
}
