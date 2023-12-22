mod ery;

use std::{
    sync::{mpsc, Arc, Mutex, RwLock},
    thread,
};

use everything_sdk::global;

use crate::tui::Event;

use self::ery::{item_to_entry, Query, QueryResults};

#[derive(Debug)]
pub struct App {
    /// event sender
    pub tui_sender: mpsc::Sender<Event>,
    /// query sender
    pub query_sender: mpsc::Sender<Query>,
    /// send back the results when query done
    pub back_recevier: Arc<Mutex<mpsc::Receiver<QueryResults>>>,
    /// query back results
    pub query_results: Arc<RwLock<QueryResults>>,
}

impl App {
    pub fn with_sender(tui_sender: mpsc::Sender<Event>) -> Self {
        let (tx_query, rx_query) = mpsc::channel::<Query>();
        let query_sender = tx_query;
        let (sync_tx_back, rx_back) = mpsc::sync_channel(0);
        let back_recevier = Arc::new(Mutex::new(rx_back));
        thread::spawn(move || {
            let mut everything = global().lock().unwrap();
            let mut searcher = everything.searcher();
            while let Ok(query) = rx_query.recv() {
                if query.search.is_empty() {
                    // do not send IPC search, return empty result
                    let empty_result = QueryResults::default();
                    sync_tx_back.send(empty_result).unwrap();
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
                        offset: query.offset,
                        number: results.num(),
                        total: results.total(),
                        request_flags: flags,
                        sort_type: results.sort_type(),
                        entrys: entrys,
                    };
                    sync_tx_back.send(query_results).unwrap();
                }
            }
        });

        Self {
            tui_sender,
            query_sender,
            back_recevier,
            query_results: Default::default(),
        }
    }

    /// trigger the SendQuery event (Everything Searching) in the terminal.
    pub fn send_query(&mut self, query_text: &str) -> anyhow::Result<()> {
        let query = Query {
            search: query_text.to_owned(),
            match_path: false,
            match_case: false,
            match_whole_word: false,
            regex: false,
            max: 512, // TODO: limit for now
            offset: 0,
            sort_type: Default::default(),
            request_flags: Default::default(),
        };
        self.query_sender.send(query)?;

        // then wait for the query results back
        let rx = Arc::clone(&self.back_recevier);
        let tui_tx = self.tui_sender.clone();
        let results_in_app = Arc::clone(&self.query_results);
        thread::spawn(move || {
            if let Ok(results) = rx.lock().unwrap().recv() {
                *results_in_app.write().unwrap() = results;
                tui_tx.send(Event::Refresh).unwrap();
            }
        });
        Ok(())
    }
}
