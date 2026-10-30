struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

fn main() {
    let mut s = Stack::new();
    s.push(1);
    s.push(2);
    s.push(3);
    println!("peek: {:?}", s.peek());
    println!("pop:  {:?}", s.pop());
    println!("len:  {}", s.len());

    // same generic type reused with a different T
    let mut words = Stack::new();
    words.push("hello");
    words.push("world");
    println!("pop:  {:?}", words.pop());
}
