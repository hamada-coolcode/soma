package main

import "fmt"

type Node struct {
	val  int
	next *Node
}

type LinkedList struct {
	head *Node
}

func (l *LinkedList) Prepend(val int) {
	l.head = &Node{val: val, next: l.head}
}

func (l *LinkedList) Len() int {
	count := 0
	for n := l.head; n != nil; n = n.next {
		count++
	}
	return count
}

func (l *LinkedList) String() string {
	s := ""
	for n := l.head; n != nil; n = n.next {
		s += fmt.Sprintf("%d -> ", n.val)
	}
	return s + "nil"
}

func main() {
	l := &LinkedList{}
	l.Prepend(3)
	l.Prepend(2)
	l.Prepend(1)
	fmt.Println("list:", l)
	fmt.Println("len: ", l.Len())
}
