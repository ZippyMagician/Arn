/*
    Just a little Deque struct a built a while back, figured I'd make use of it here
    Not necessary at all, but this project is just a test so why not?
*/

#![allow(dead_code)]

use std::fmt::{Debug, Error, Formatter};

/// Passed into the `queue` and `dequeue` functions in order to determine
/// which edge of the stack to push the entry into
pub enum Position {
    /// Marks the bottom of the stack
    FRONT,

    /// Marks the top of the stack
    BACK,
}

/// A `Vec<T>` wrapper struct that can take and yield arguments from both sides. Takes a generic, `T`, which allows for various forms of data to be stored.
/// ```
/// let deque: Deque<i128> = Deque::with_allocation(10);
///
/// deque.push_front(38);
/// deque.push_front(23);
/// deque.push_back(103);
///
/// assert_eq!(deque.dequeue(Position::FRONT), 23);
/// assert_eq!(deque.dequeue(Position::BACK), 103);
/// ```
pub struct Deque<T> {
    /// The main stack that stores information inside the `Deque`. Think of
    /// the `Deque` as a wrapper for a Vec
    buffer: Vec<T>,

    /// Marks how large the `Vec` inside the `Deque` will be, and will resize by this amount
    /// when that limit is reached.
    allocation_size: usize,
}

impl<T> Deque<T>
where
    T: Clone + PartialEq,
{
    /// Create a new instance of the `Deque` with the default allocation size of 250
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(250),
            allocation_size: 250,
        }
    }

    /// Create a new instance of the `Deque` with the specified capacity
    pub fn with_allocation(size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(size),
            allocation_size: size,
        }
    }

    /// Private function that is called by `push_front` and `push_back`.
    /// Checks if the Deque needs to be resized to make room for more entries, and if so expands it.
    fn resize(&mut self) {
        if self.buffer.len() % self.allocation_size == 0 {
            self.buffer.reserve(self.allocation_size);
        }
    }

    /// Returns the index of an entry
    pub fn find_entry(&self, entry: T) -> Option<usize> {
        let mut i = 0;
        loop {
            if i >= self.buffer.len() {
                return None;
            }

            if *self.buffer.get(i).unwrap() == entry {
                return Some(i);
            }

            i += 1;
        }
    }

    /// Removes certain entry from queue
    pub fn remove(&mut self, index: usize) {
        self.buffer.remove(index);
    }

    // Get length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    // Convert to Vec
    pub fn to_vec(&self) -> Vec<T> {
        self.buffer.clone()
    }

    /// Pushes an entry to the bottom of the stack
    pub fn push_front(&mut self, item: T) {
        self.resize();
        self.buffer.push(item);
        self.buffer.rotate_right(1);
    }

    /// Pushes an entry to the top of the stack
    pub fn push_back(&mut self, item: T) {
        self.resize();
        self.buffer.push(item);
    }

    /// Returns true if the deque is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.len() == 0
    }

    /// Returns the top item of the deque
    pub fn get_top(&self) -> &T {
        self.buffer.get(self.buffer.len() - 1).unwrap()
    }

    /// Pushes an entry to either the top or bottom of a stack, depending on the
    /// position passed (see `Position`)
    pub fn queue(&mut self, item: T, pos: Position) {
        match pos {
            Position::FRONT => {
                self.push_front(item);
            }
            Position::BACK => {
                self.push_back(item);
            }
        }
    }

    /// Removes an entry from the top or bottom of the stack, depending on the
    /// position passed. Will return an Option containing the entry.
    pub fn dequeue(&mut self, pos: Position) -> Option<T> {
        match pos {
            Position::FRONT => {
                self.buffer.rotate_right(1);
                self.buffer.pop()
            }
            Position::BACK => self.buffer.pop(),
        }
    }
}

/// Allows users to easily debug what is inside a Deque
/// ```
/// println!("{:?}", deque);
/// ```
impl<T> Debug for Deque<T>
where
    T: Debug,
{
    fn fmt(&self, format: &mut Formatter<'_>) -> Result<(), Error> {
        format.write_fmt(format_args!("{:?}", self.buffer))
    }
}
