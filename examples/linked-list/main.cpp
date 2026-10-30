#include <iostream>

// Raw owning pointers: manual new/delete (the destructor frees the chain)
struct Node {
    int val;
    Node* next;
    Node(int v, Node* n = nullptr) : val(v), next(n) {}
};

class LinkedList {
    Node* head = nullptr;

public:
    ~LinkedList() {
        while (head) {
            Node* temp = head;
            head = head->next;
            delete temp;
        }
    }

    void prepend(int val) { head = new Node(val, head); }

    int len() const {
        int count = 0;
        for (Node* n = head; n; n = n->next) count++;
        return count;
    }

    friend std::ostream& operator<<(std::ostream& os, const LinkedList& l) {
        for (Node* n = l.head; n; n = n->next) os << n->val << " -> ";
        return os << "nil";
    }
};

int main() {
    LinkedList l;
    l.prepend(3);
    l.prepend(2);
    l.prepend(1);
    std::cout << "list: " << l << "\n";
    std::cout << "len:  " << l.len() << "\n";
}
