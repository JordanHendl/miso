use std::collections::VecDeque;

#[derive(Default)]
pub struct DeletionQueue<T> {
    queue: VecDeque<Box<dyn FnOnce() -> T + Send + 'static>>,
}

impl<T: Clone> Clone for DeletionQueue<T> {
    fn clone(&self) -> Self {
        Self {
            queue: Default::default(),
        }
    }
}

impl<T> DeletionQueue<T> {
    /// Creates a new, empty `DeletionQueue`.
    pub fn new() -> Self {
        DeletionQueue {
            queue: VecDeque::new(),
        }
    }

    /// Adds a deletion operation to the queue.
    ///
    /// # Arguments
    /// * `operation` - A closure or function that takes no arguments and returns a value of type `T`.
    pub fn push<F>(&mut self, operation: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        self.queue.push_back(Box::new(operation));
    }

    /// Processes all operations in the queue and clears it.
    ///
    /// Returns a `Vec<T>` containing the results of all processed operations.
    pub fn delete_all(&mut self) -> Vec<T> {
        let mut results = Vec::new();

        while let Some(operation) = self.queue.pop_front() {
            results.push(operation());
        }

        results
    }

    /// Checks if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the number of operations currently in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
