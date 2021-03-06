use std::process::exit;

use libimagdiary::diary::Diary;
use libimagentryview::viewer::Viewer;
use libimagentryview::builtin::plain::PlainViewer;
use libimagrt::runtime::Runtime;
use libimagerror::trace::trace_error;

use util::get_diary_name;

pub fn view(rt: &Runtime) {
    let diaryname = get_diary_name(rt);
    if diaryname.is_none() {
        warn!("No diary name");
        exit(1);
    }
    let diaryname = diaryname.unwrap();
    let diary = Diary::open(rt.store(), &diaryname[..]);
    let show_header = rt.cli().subcommand_matches("view").unwrap().is_present("show-header");

    match diary.entries() {
        Ok(entries) => {
            let pv = PlainViewer::new(show_header);
            for entry in entries.into_iter().filter_map(Result::ok) {
                let id = entry.diary_id();
                println!("{} :\n", id);
                pv.view_entry(&entry);
                println!("\n---\n");
            }
        },
        Err(e) => trace_error(&e),
    }
}

