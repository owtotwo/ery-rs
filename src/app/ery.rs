use std::{ffi::OsString, path::PathBuf};

use everything_sdk::{EverythingItem, RequestFlags, SortType};

#[derive(Debug)]
pub struct Query {
    pub search: String,
    pub match_path: bool,
    pub match_case: bool,
    pub match_whole_word: bool,
    pub regex: bool,
    pub max: u32,
    pub offset: u32,
    pub sort_type: SortType,
    pub request_flags: RequestFlags,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            search: "".to_string(),
            match_path: false,
            match_case: false,
            match_whole_word: false,
            regex: false,
            max: u32::MAX,
            offset: 0,
            sort_type: Default::default(),
            request_flags: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct QueryResults {
    pub search: OsString,
    pub offset: u32,
    pub number: u32,
    pub total: u32,
    pub request_flags: RequestFlags,
    pub sort_type: SortType,
    pub entrys: Vec<QueryEntry>,
}

#[derive(Debug)]
pub struct QueryEntry {
    pub index: u32,
    pub is_volume: bool,
    pub is_folder: bool,
    pub is_file: bool,
    pub filename: Option<OsString>,
    pub path: Option<PathBuf>,
    pub filepath: Option<PathBuf>,
    pub full_path_name: Option<PathBuf>,
    pub extension: Option<OsString>,
    pub size: Option<u64>,
    pub date_created: Option<u64>,
    pub date_modified: Option<u64>,
    pub date_accessed: Option<u64>,
    pub attributes: Option<u32>,
    pub file_list_filename: Option<OsString>,
    pub run_count: Option<u32>,
    pub date_run: Option<u64>,
    pub date_recently_changed: Option<u64>,
    pub highlighted_filename: Option<OsString>,
    pub highlighted_path: Option<OsString>,
    pub highlighted_full_path_and_filename: Option<OsString>,
}

pub fn item_to_entry(item: EverythingItem<'_>, request_flags: RequestFlags) -> QueryEntry {
    let index = item.index();
    let is_volume = item.is_volume();
    let is_folder = item.is_folder();
    let is_file = item.is_file();

    let filename = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_FILE_NAME)
        .then(|| item.filename().unwrap());
    let path = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_PATH)
        .then(|| item.path().unwrap());
    let filepath = request_flags
        .contains(
            RequestFlags::EVERYTHING_REQUEST_PATH | RequestFlags::EVERYTHING_REQUEST_FILE_NAME,
        )
        .then(|| item.filepath().unwrap());
    let full_path_name = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_FULL_PATH_AND_FILE_NAME)
        .then(|| item.full_path_name(None).unwrap());
    let extension = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_EXTENSION)
        .then(|| item.extension().unwrap());
    let size = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_SIZE)
        .then(|| item.size().unwrap());
    let date_created = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_DATE_CREATED)
        .then(|| item.date_created().unwrap());
    let date_modified = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_DATE_MODIFIED)
        .then(|| item.date_modified().unwrap());
    let date_accessed = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_DATE_ACCESSED)
        .then(|| item.date_accessed().unwrap());
    let attributes = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES)
        .then(|| item.attributes().unwrap());
    let file_list_filename = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_FILE_LIST_FILE_NAME)
        .then(|| item.file_list_filename().unwrap());
    let run_count = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_RUN_COUNT)
        .then(|| item.run_count().unwrap());
    let date_run = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_DATE_RUN)
        .then(|| item.date_run().unwrap());
    let date_recently_changed = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_DATE_RECENTLY_CHANGED)
        .then(|| item.date_recently_changed().unwrap());
    let highlighted_filename = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_HIGHLIGHTED_FILE_NAME)
        .then(|| item.highlighted_filename().unwrap());
    let highlighted_path = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_HIGHLIGHTED_PATH)
        .then(|| item.highlighted_path().unwrap());
    let highlighted_full_path_and_filename = request_flags
        .contains(RequestFlags::EVERYTHING_REQUEST_HIGHLIGHTED_FULL_PATH_AND_FILE_NAME)
        .then(|| item.highlighted_full_path_and_filename().unwrap());

    QueryEntry {
        index,
        is_volume,
        is_folder,
        is_file,
        filename,
        path,
        filepath,
        full_path_name,
        extension,
        size,
        date_created,
        date_modified,
        date_accessed,
        attributes,
        file_list_filename,
        run_count,
        date_run,
        date_recently_changed,
        highlighted_filename,
        highlighted_path,
        highlighted_full_path_and_filename,
    }
}
