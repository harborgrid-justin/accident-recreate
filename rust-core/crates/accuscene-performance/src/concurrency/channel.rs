//! High-performance channels using flume

use crate::error::{PerformanceError, Result};
use flume::{Receiver as FlumeReceiver, Sender as FlumeSender};
use std::time::Duration;

/// High-performance sender
pub struct Sender<T> {
    inner: FlumeSender<T>,
}

impl<T> Sender<T> {
    /// Send a value (blocking)
    pub fn send(&self, value: T) -> Result<()> {
        self.inner.send(value)?;
        Ok(())
    }

    /// Try to send a value (non-blocking)
    pub fn try_send(&self, value: T) -> Result<()> {
        self.inner.try_send(value).map_err(|_| PerformanceError::ChannelSend)?;
        Ok(())
    }

    /// Send with timeout
    pub fn send_timeout(&self, value: T, timeout: Duration) -> Result<()> {
        self.inner
            .send_timeout(value, timeout)
            .map_err(|_| PerformanceError::Timeout {
                duration_ms: timeout.as_millis() as u64,
            })?;
        Ok(())
    }

    /// Check if channel is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if channel is full
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Get number of items in channel
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// High-performance receiver
pub struct Receiver<T> {
    inner: FlumeReceiver<T>,
}

impl<T> Receiver<T> {
    /// Receive a value (blocking)
    pub fn recv(&self) -> Result<T> {
        Ok(self.inner.recv()?)
    }

    /// Try to receive a value (non-blocking)
    pub fn try_recv(&self) -> Result<T> {
        self.inner.try_recv().map_err(|_| PerformanceError::ChannelReceive)
    }

    /// Receive with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T> {
        self.inner
            .recv_timeout(timeout)
            .map_err(|_| PerformanceError::Timeout {
                duration_ms: timeout.as_millis() as u64,
            })
    }

    /// Create an iterator over received values
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.inner.iter()
    }

    /// Try to iterate over available values
    pub fn try_iter(&self) -> impl Iterator<Item = T> + '_ {
        self.inner.try_iter()
    }

    /// Check if channel is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get number of items in channel
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Create a bounded channel
pub fn bounded<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::bounded(capacity);
    (Sender { inner: tx }, Receiver { inner: rx })
}

/// Create an unbounded channel
pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::unbounded();
    (Sender { inner: tx }, Receiver { inner: rx })
}

/// Multiple producer, single consumer channel
pub struct MpscChannel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> MpscChannel<T> {
    /// Create a new MPSC channel
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self { sender, receiver }
    }

    /// Create unbounded MPSC channel
    pub fn unbounded() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    /// Get a sender
    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }

    /// Get the receiver
    pub fn receiver(&self) -> &Receiver<T> {
        &self.receiver
    }

    /// Split into sender and receiver
    pub fn split(self) -> (Sender<T>, Receiver<T>) {
        (self.sender, self.receiver)
    }
}

/// Single producer, multiple consumer channel
pub struct SpmcChannel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> SpmcChannel<T> {
    /// Create a new SPMC channel
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self { sender, receiver }
    }

    /// Create unbounded SPMC channel
    pub fn unbounded() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    /// Get the sender
    pub fn sender(&self) -> &Sender<T> {
        &self.sender
    }

    /// Get a receiver
    pub fn receiver(&self) -> Receiver<T> {
        self.receiver.clone()
    }

    /// Split into sender and receiver
    pub fn split(self) -> (Sender<T>, Receiver<T>) {
        (self.sender, self.receiver)
    }
}

/// Broadcast channel for multiple subscribers
pub struct BroadcastChannel<T: Clone> {
    senders: Vec<Sender<T>>,
}

impl<T: Clone + Send + 'static> BroadcastChannel<T> {
    /// Create a new broadcast channel
    pub fn new(num_receivers: usize, capacity: usize) -> (Sender<T>, Vec<Receiver<T>>) {
        let mut receivers = Vec::new();
        let mut senders = Vec::new();

        for _ in 0..num_receivers {
            let (tx, rx) = bounded(capacity);
            senders.push(tx);
            receivers.push(rx);
        }

        let broadcast_sender = BroadcastSender { senders };

        // Create a forwarding channel
        let (tx, rx) = bounded(capacity);

        // Spawn forwarding task
        std::thread::spawn(move || {
            for msg in rx.iter() {
                broadcast_sender.send(msg).ok();
            }
        });

        (tx, receivers)
    }
}

/// Broadcast sender
struct BroadcastSender<T: Clone + Send> {
    senders: Vec<Sender<T>>,
}

impl<T: Clone + Send> BroadcastSender<T> {
    fn send(&self, value: T) -> Result<()> {
        for (i, sender) in self.senders.iter().enumerate() {
            let v = if i == self.senders.len() - 1 {
                value.clone()
            } else {
                value.clone()
            };
            sender.send(v)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_channel() {
        let (tx, rx) = bounded(10);

        tx.send(42).unwrap();
        tx.send(100).unwrap();

        assert_eq!(rx.recv().unwrap(), 42);
        assert_eq!(rx.recv().unwrap(), 100);
    }

    #[test]
    fn test_unbounded_channel() {
        let (tx, rx) = unbounded();

        for i in 0..1000 {
            tx.send(i).unwrap();
        }

        for i in 0..1000 {
            assert_eq!(rx.recv().unwrap(), i);
        }
    }

    #[test]
    fn test_try_send_recv() {
        let (tx, rx) = bounded(2);

        assert!(tx.try_send(1).is_ok());
        assert!(tx.try_send(2).is_ok());

        assert_eq!(rx.try_recv().unwrap(), 1);
        assert_eq!(rx.try_recv().unwrap(), 2);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_mpsc_channel() {
        let channel = MpscChannel::new(10);

        let tx1 = channel.sender();
        let tx2 = channel.sender();

        tx1.send(1).unwrap();
        tx2.send(2).unwrap();

        assert_eq!(channel.receiver().recv().unwrap(), 1);
        assert_eq!(channel.receiver().recv().unwrap(), 2);
    }

    #[test]
    fn test_spmc_channel() {
        let channel = SpmcChannel::new(10);

        let rx1 = channel.receiver();
        let rx2 = channel.receiver();

        channel.sender().send(42).unwrap();

        // One receiver will get it
        let result = rx1.try_recv().or_else(|_| rx2.try_recv());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_channel_len() {
        let (tx, rx) = bounded(10);

        assert_eq!(tx.len(), 0);
        assert!(tx.is_empty());

        tx.send(1).unwrap();
        tx.send(2).unwrap();

        assert_eq!(rx.len(), 2);
        assert!(!rx.is_empty());
    }

    #[test]
    fn test_recv_timeout() {
        let (_, rx) = bounded::<i32>(10);

        let result = rx.recv_timeout(Duration::from_millis(10));
        assert!(result.is_err());
    }
}
