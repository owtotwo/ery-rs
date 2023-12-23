mod ery;

use std::{
    sync::{mpsc, Arc, Mutex, RwLock},
    thread,
};

use everything_sdk::{global, FileInfoType, SortType};

use crate::tui::Event;

use self::ery::{item_to_entry, Query, QueryResults};

#[derive(Debug)]
pub struct App {
    /// everything status
    pub status: Status,
    /// event sender
    pub tui_sender: mpsc::Sender<Event>,
    /// query sender
    pub query_sender: mpsc::Sender<Query>,
    /// send back the results when query done
    pub back_recevier: Arc<Mutex<mpsc::Receiver<QueryResults>>>,
    /// query back results
    pub query_results: Arc<RwLock<QueryResults>>,
}

#[derive(Debug)]
pub struct Status {
    pub is_db_loaded: bool,

    /// Everything version format: `<major>.<minor>.<revision>.<build>`.
    pub version: (u32, u32, u32, u32),

    pub is_admin: bool,
    pub is_appdata: bool,

    pub is_file_size_indexed: bool,
    pub is_folder_size_indexed: bool,
    pub is_date_created_indexed: bool,
    pub is_date_modified_indexed: bool,
    pub is_date_accessed_indexed: bool,
    pub is_attributes_indexed: bool,

    pub is_size_fast_sort: bool,
    pub is_date_created_fast_sort: bool,
    pub is_date_modified_fast_sort: bool,
    pub is_date_accessed_fast_sort: bool,
    pub is_attributes_fast_sort: bool,
    pub is_path_fast_sort: bool,
    pub is_extension_fast_sort: bool,
}

impl App {
    pub fn with_sender(tui_sender: mpsc::Sender<Event>) -> Self {
        let status = App::load_status().unwrap();
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
            status: status,
            tui_sender,
            query_sender,
            back_recevier,
            query_results: Default::default(),
        }
    }

    fn load_status() -> anyhow::Result<Status> {
        let everything = global().try_lock().unwrap();
        let is_db_loaded = everything.is_db_loaded()?;
        let (major, minor, revision, build, _target) = everything.version()?;
        let version = (major, minor, revision, build);
        let is_admin = everything.is_admin()?;
        let is_appdata = everything.is_appdata()?;
        let is_file_size_indexed =
            everything.is_file_info_indexed(FileInfoType::EVERYTHING_IPC_FILE_INFO_FILE_SIZE)?;
        let is_folder_size_indexed =
            everything.is_file_info_indexed(FileInfoType::EVERYTHING_IPC_FILE_INFO_FOLDER_SIZE)?;
        let is_date_created_indexed =
            everything.is_file_info_indexed(FileInfoType::EVERYTHING_IPC_FILE_INFO_DATE_CREATED)?;
        let is_date_modified_indexed = everything
            .is_file_info_indexed(FileInfoType::EVERYTHING_IPC_FILE_INFO_DATE_MODIFIED)?;
        let is_date_accessed_indexed = everything
            .is_file_info_indexed(FileInfoType::EVERYTHING_IPC_FILE_INFO_DATE_ACCESSED)?;
        let is_attributes_indexed =
            everything.is_file_info_indexed(FileInfoType::EVERYTHING_IPC_FILE_INFO_ATTRIBUTES)?;
        let is_size_fast_sort = everything.is_fast_sort(SortType::EVERYTHING_SORT_SIZE_ASCENDING)?;
        let is_date_created_fast_sort =
            everything.is_fast_sort(SortType::EVERYTHING_SORT_DATE_CREATED_ASCENDING)?;
        let is_date_modified_fast_sort =
            everything.is_fast_sort(SortType::EVERYTHING_SORT_DATE_MODIFIED_ASCENDING)?;
        let is_date_accessed_fast_sort =
            everything.is_fast_sort(SortType::EVERYTHING_SORT_DATE_ACCESSED_ASCENDING)?;
        let is_attributes_fast_sort =
            everything.is_fast_sort(SortType::EVERYTHING_SORT_ATTRIBUTES_ASCENDING)?;
        let is_path_fast_sort = everything.is_fast_sort(SortType::EVERYTHING_SORT_PATH_ASCENDING)?;
        let is_extension_fast_sort =
            everything.is_fast_sort(SortType::EVERYTHING_SORT_EXTENSION_ASCENDING)?;
        let status = Status {
            is_db_loaded,
            version,
            is_admin,
            is_appdata,
            is_file_size_indexed,
            is_folder_size_indexed,
            is_date_created_indexed,
            is_date_modified_indexed,
            is_date_accessed_indexed,
            is_attributes_indexed,
            is_size_fast_sort,
            is_date_created_fast_sort,
            is_date_modified_fast_sort,
            is_date_accessed_fast_sort,
            is_attributes_fast_sort,
            is_path_fast_sort,
            is_extension_fast_sort,
        };

        Ok(status)
    }

    /// trigger the SendQuery event (Everything Searching) in the terminal.
    pub fn send_query(&mut self, query_text: &str) -> anyhow::Result<()> {
        let query = Query {
            search: query_text.to_owned(),
            match_path: false,
            match_case: false,
            match_whole_word: false,
            regex: false,
            max: 512, // TODO: limit for now, maybe dynamic loading in the future.
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
