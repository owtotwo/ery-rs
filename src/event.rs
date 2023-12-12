use crate::ery::QueryResults;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use everything_sdk::error::EverythingError;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;

#[derive(Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press/release/repeat.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
    /// App refresh request.
    Refresh,
    /// Everything searching by sending query.
    SendQuery,
    /// Everything give back the query results.
    QueryBack(QueryResults),
    /// Error when sending query.
    QueryError(EverythingError),
}

#[derive(Debug)]
pub struct EventHandler {
    pub sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    _handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn with_tick(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let _handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("failed to poll events") {
                        match event::read().expect("failed to read the event") {
                            CrosstermEvent::FocusGained => Ok(()),
                            CrosstermEvent::FocusLost => Ok(()),
                            CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                            CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                            CrosstermEvent::Paste(_) => Ok(()),
                            CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                        }
                        .expect("failed to send terminal event")
                    }

                    if last_tick.elapsed() >= tick_rate {
                        // it seems that we may not need the tick, just do nothing when user do nothing
                        // sender.send(Event::Tick).expect("failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            _handler,
        }
    }

    /// Receive the next event from the event handler thread.
    ///
    /// block the current thread and wait the next event
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}
