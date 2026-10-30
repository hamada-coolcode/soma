#include <iostream>
#include <optional>
#include <string>
#include <vector>

template <typename T>
class Stack {
    std::vector<T> items;

public:
    void push(const T& item) { items.push_back(item); }

    std::optional<T> pop() {
        if (items.empty()) return std::nullopt;
        T item = items.back();
        items.pop_back();
        return item;
    }

    std::optional<T> peek() const {
        if (items.empty()) return std::nullopt;
        return items.back();
    }

    std::size_t len() const { return items.size(); }
};

int main() {
    Stack<int> s;
    s.push(1);
    s.push(2);
    s.push(3);
    std::cout << "peek: " << s.peek().value() << "\n";
    std::cout << "pop:  " << s.pop().value() << "\n";
    std::cout << "len:  " << s.len() << "\n";

    // same template reused with a different T
    Stack<std::string> words;
    words.push("hello");
    words.push("world");
    std::cout << "pop:  " << words.pop().value() << "\n";
}
