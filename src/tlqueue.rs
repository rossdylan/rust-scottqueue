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
    /// Create a new Node<T> struct.
    /// This is only used internally by the Queue functions.
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
    /// Create a new scottqueue::tlqueue::Queue<T> struct
    /// Starts out empty, with a single Node containing `None`
    ///
    /// # Examples
    ///
    /// ```
    /// use scottqueue::tlqueue::Queue;
    /// let q: Queue<i64> = Queue::new();
    /// ```
    pub fn new() -> Queue<T> {
        let null_node : *mut Node<T> = Node::new(None);
        Queue {
            head: Mutex::new(null_node),
            tail: Mutex::new(null_node),
        }
    }

    /// Push a value into the Queue.
    /// Internally this creates a new Node<T> and sets its value
    /// to the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scottqueue::tlqueue::Queue;
    /// let q: Queue<i64> = Queue::new();
    /// q.push(12);
    /// ```
    pub fn push(&self, value : T) {
        let new_node : *mut Node<T> = Node::new(Some(value));
        let mut tail = self.tail.lock().unwrap();
        unsafe {
            (**tail).next = new_node;
            *tail = new_node;
        }
    }

    /// pop a value from the Queue.
    /// Internally this grabs the head of the queue, and returns the value.
    /// It then removes that node and sets the head to the next node.
    ///
    ///  Examples
    ///
    /// ```
    /// use scottqueue::tlqueue::Queue;
    /// let q: Queue<i64> = Queue::new();
    /// q.push(12);
    /// println!("Result!: {}", q.pop().unwrap());
    /// ```
    pub fn pop(&self) -> Option<T> {
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
    extern crate test;

    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::collections::HashSet;
    use std::thread;
    use self::test::Bencher;

    #[test]
    fn test_single_item() {
        let q: super::Queue<i64> = super::Queue::new();
        q.push(1);
        assert_eq!(q.pop().unwrap(), 1);
    }

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
        for _ in 0..100 {
            _test_threads();
        }
    }

    fn _test_threads() {
        let (tx, rx) = channel();
        let nthreads = 20;
        let nmsgs = 10000;

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
                for _ in 0..(nmsgs/nthreads) {
                    tx.send(q.pop().unwrap()).unwrap();
                }
                drop(tx);
            });
        }
        for _ in 0..nmsgs {
            let msg = rx.recv().unwrap();
            assert!(!end_set.contains(&msg));
            end_set.insert(msg);

        }
        assert_eq!(end_set, start_set);
    }

    #[bench]
    fn push_bench(b: &mut Bencher) {
        let q = super::Queue::new();
        b.iter(|| q.push(0));
    }

    #[bench]
    fn pop_bench(b: &mut Bencher) {
        let q = super::Queue::new();
        for i in 0..100000 {
            q.push(i);
        }
        b.iter(|| q.pop());
    }

    #[bench]
    fn channel_send_bench(b: &mut Bencher) {
        let (tx, rx) = channel();
        b.iter(|| tx.send(0).unwrap());
        drop(tx);
        drop(rx);
    }

    #[bench]
    fn channel_recv_bench(b: &mut Bencher) {
        let (tx, rx) = channel();
        for i in 0..100000 {
            tx.send(i).unwrap();
        }
        b.iter(|| rx.recv().unwrap());
        drop(tx);
        drop(rx);
    }

}
