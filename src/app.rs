use std::{sync::mpsc, thread};

use everything_sdk::global;
use tui_textarea::TextArea;

use crate::{
    ery::{item_to_entry, Query, QueryResults},
    event::Event,
};

#[derive(Debug)]
pub struct App<'a> {
    /// running state
    pub is_running: bool,
    /// event sender
    pub event_sender: mpsc::Sender<Event>,
    /// Event handler thread for Everything-SDK.
    pub query_handler: thread::JoinHandle<()>,
    /// query sender
    pub query_sender: mpsc::Sender<Query>,
    /// query back results
    pub query_results: QueryResults,
    /// counter for temporary test
    pub counter: u64,
    /// tick count
    pub tick: u32,
    /// refresh count
    pub refresh: u32,
    /// search input
    pub textarea: TextArea<'a>,
}

impl App<'_> {
    pub fn with_sender(sender: mpsc::Sender<Event>) -> Self {
        let (tx, rx) = mpsc::channel::<Query>();
        let inner_sender = sender.clone();
        let everything_handler = thread::spawn(move || {
            let mut everything = global().lock().unwrap();
            let mut searcher = everything.searcher();
            while let Ok(query) = rx.recv() {
                if query.search.is_empty() {
                    // do not send IPC search, return empty result
                    let empty_result = QueryResults::default();
                    inner_sender.send(Event::QueryBack(empty_result)).unwrap();
                } else {
                    searcher
                        .set_search(query.search)
                        .set_match_path(query.match_path)
                        .set_match_case(query.match_case)
                        .set_match_whole_word(query.match_whole_word)
                        .set_regex(query.regex)
                        .set_max(query.max)
                        .set_offset(query.offset)
                        .set_sort(query.sort_type)
                        .set_request_flags(query.request_flags);
                    let search_text = searcher.get_search();
                    let results = searcher.query();
                    let flags = results.request_flags();
                    let entrys: Vec<_> = results.iter().map(|i| item_to_entry(i, flags)).collect();
                    let query_results = QueryResults {
                        search: search_text,
                        number: results.num(),
                        total: results.total(),
                        request_flags: flags,
                        sort_type: results.sort_type(),
                        entrys: entrys,
                    };
                    inner_sender.send(Event::QueryBack(query_results)).unwrap();
                }
            }
        });
        Self {
            is_running: true,
            event_sender: sender,
            query_handler: everything_handler,
            query_sender: tx,
            query_results: Default::default(),
            counter: 0,
            tick: 0,
            refresh: 0,
            textarea: Default::default(),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.tick += 1;
    }

    /// Tell that the app is over.
    pub fn quit(&mut self) {
        self.is_running = false;
    }

    /// Set the text in search bar by yank and paste
    pub fn set_search_text(&mut self, text: &str) {
        let old = self.textarea.yank_text();
        self.textarea.set_yank_text(text);
        self.textarea.paste();
        self.textarea.set_yank_text(old);
    }

    /// trigger the SendQuery event (Everything Searching) in the terminal.
    pub fn send_query(&mut self) {
        self.event_sender.send(Event::SendQuery).unwrap();
    }

    /// trigger Refresh Event to tui for new frame renderring (e.g. when app data updated)
    pub fn refresh_now(&self) {
        self.event_sender.send(Event::Refresh).unwrap();
    }

    pub fn increment_counter(&mut self) {
        self.counter += 1;
    }

    pub fn decrement_counter(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
        }
    }
}
