use text::{internal::Internal, text::Text, Leaf, Node};

fn main() {
    dbg!(std::mem::size_of::<Text>());
    dbg!(std::mem::size_of::<Node>());
    dbg!(std::mem::size_of::<Internal>());
    dbg!(std::mem::size_of::<Leaf>());
}
