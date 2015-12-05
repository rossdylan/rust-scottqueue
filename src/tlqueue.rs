use std::ptr;
use std::sync::{Mutex};
use std::iter::FromIterator;

struct Node<T> {
    value: Option<T>,
    next: *mut Node<T>
}

pub struct Queue<T> {
    head: Mutex<*mut Node<T>>,
    tail: Mutex<*mut Node<T>>
}

impl<T> Node<T> {
    fn new(v: Option<T>) -> *mut Node<T> {
        Box::into_raw(Box::new(Node {
            next: ptr::null_mut(),
            value: v
        }))
    }
}

unsafe impl<T> Sync for Queue<T> {}
unsafe impl<T> Send for Queue<T> {}

impl<T> Queue<T> {
    fn new() -> Queue<T> {
        let null_node : *mut Node<T> = Node::new(None);
        Queue {
            head: Mutex::new(null_node),
            tail: Mutex::new(null_node),
        }
    }

    fn push(&self, value : T) {
        let new_node : *mut Node<T> = Node::new(Some(value));
        let mut tail = self.tail.lock().unwrap();
        unsafe {
            (**tail).next = new_node;
            *tail = new_node;
        }
    }

    fn pop(&self) -> Option<T> {
        let mut head = self.head.lock().unwrap();
        unsafe {
            if (**head).next.is_null() { // is queue empty?
                return None;
            }
            let value = (*(**head).next).value.take().unwrap();
            let _: Box<Node<T>> = Box::from_raw(*head); // ? make sure that shit is deleted?
            *head = (**head).next;
            return Some(value);
        }
    }
}
impl<T> Iterator for Queue<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.pop()
    }
}

impl<A> FromIterator<A> for Queue<A> {
    fn from_iter<T>(iterator: T) -> Self where T: IntoIterator<Item=A> {
        let q = Queue::new();
        for item in iterator {
            q.push(item);
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::collections::HashSet;
    use std::thread;

    #[test]
    fn test_queue() {
        let values = &vec![1, 2, 3, 4, 5];
        let q: super::Queue<i64> = super::Queue::new();
        for value in values {
            q.push(*value);
        }
        let mut results: Vec<i64> = vec![];
        for _ in 0..5 { 
            results.push(q.pop().unwrap());
        }
        for res in &results {
            println!("Result: {}", res);
        }
        assert_eq!(&results, values);
    }

    #[test]
    fn test_iterator() {
        let values = &vec![1, 2, 3, 4, 5];
        let q: super::Queue<i64> = super::Queue::new();
        for value in values {
            q.push(*value);
        }
        let results : Vec<i64> = q.collect();
        for res in &results {
            println!("Result: {}", res);
        }
        assert_eq!(&results, values);
    }

    #[test]
    fn test_from_iterator() {
        let values = &vec![1, 2, 3, 4, 5];
        let q: super::Queue<i64> = values.clone().into_iter().collect();
        let results : Vec<i64> = q.collect();
        assert_eq!(&results, values);
    }

    #[test]
    fn test_threads() {
        let (tx, rx) = channel();
        let nthreads = 8;
        let nmsgs = 5000;

        let q = Arc::new(super::Queue::new());
        let mut start_set = HashSet::new();
        let mut end_set = HashSet::new();
        for i in 0..nmsgs {
            q.push(i);
            start_set.insert(i);
        }
        for _ in 0..nthreads {
            let tx = tx.clone();
            let q = q.clone();
            thread::spawn(move|| {
                for _ in 0..(nmsgs/8) {
                    tx.send(q.pop().unwrap()).unwrap();
                }
                drop(tx);
            });
        }
        for _ in 0..nmsgs {
            end_set.insert(rx.recv().unwrap());
        }
        assert_eq!(end_set, start_set);
    }
}
