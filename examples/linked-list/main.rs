#[derive(Debug)]
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}

impl<T> List<T> {
    fn new() -> Self {
        List::Nil
    }

    fn prepend(self, val: T) -> Self {
        List::Cons(val, Box::new(self))
    }

    fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(_, tail) => 1 + tail.len(),
        }
    }
}

fn main() {
    // persistent/immutable list: each prepend builds a new head
    let list = List::new().prepend(3).prepend(2).prepend(1);
    println!("list: {:?}", list);
    println!("len:  {}", list.len());
}
