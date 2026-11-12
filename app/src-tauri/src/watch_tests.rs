use super::*;
use notify::event::{
    AccessKind, AccessMode, CreateKind, DataChange, Event, EventKind, ModifyKind, RemoveKind,
    RenameMode,
};
use std::collections::BTreeMap;
use std::fs;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use tauri::test::{MockRuntime, mock_app};
use tauri::{App, Listener};
use twinova_project::Project;
use twinova_project::sources::{DiskSources, Kind};
use twinova_project::storage::open_folder;

use crate::project_commands::open_project;
use crate::test_fixtures::copied;

/// Long enough for a loaded CI runner; a passing wait returns as soon as the event arrives.
const WAIT: Duration = Duration::from_secs(10);
/// How long a test waits to see that nothing more arrives: several quiet periods.
const NOTHING: Duration = Duration::from_millis(1_500);

fn project(folder: &str) -> Project {
    open_folder(Path::new(folder)).unwrap().opened.project
}

/// A watch whose quiet periods arrive on the returned channel.
fn watching(folder: &str) -> (SourceWatch, Receiver<PathBuf>) {
    let (tx, rx) = mpsc::channel();
    let watch = SourceWatch::new(move |folder| tx.send(folder.to_path_buf()).unwrap());
    watch.follow(Some(Path::new(folder))).unwrap();
    (watch, rx)
}

fn main_st(folder: &str) -> PathBuf {
    Path::new(folder).join("plc/Main.st")
}

fn event(kind: EventKind, path: &str) -> Event {
    Event::new(kind).add_path(PathBuf::from(path))
}

#[test]
fn only_a_change_outside_twinova_s_own_state_is_relevant() {
    let folder = Path::new("/p/line");
    let written = EventKind::Modify(ModifyKind::Data(DataChange::Any));
    for path in [
        "/p/line/plc/Main.st",
        "/p/line/components/x.st",
        "/p/line/plc/.Main.st.swp",
    ] {
        assert!(relevant(folder, &event(written, path)), "{path}");
    }
    assert!(relevant(
        folder,
        &event(EventKind::Create(CreateKind::File), "/p/line/plc/Main.st")
    ));
    assert!(relevant(
        folder,
        &event(EventKind::Remove(RemoveKind::Any), "/p/line/plc/Main.st")
    ));
    assert!(relevant(
        folder,
        &event(EventKind::Any, "/p/line/plc/Main.st")
    ));
    for path in [
        "/p/line/.twinova/previous/plc/Main.st",
        "/p/line/.twinova",
        "/p/other/plc/Main.st",
    ] {
        assert!(!relevant(folder, &event(written, path)), "{path}");
    }
    // Reading a file is not changing it: Linux reports reads, Twinova's own included (measured 2026-11-12).
    for read in [
        EventKind::Access(AccessKind::Open(AccessMode::Any)),
        EventKind::Access(AccessKind::Close(AccessMode::Read)),
    ] {
        assert!(!relevant(folder, &event(read, "/p/line/plc/Main.st")));
    }
    let renamed = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
        .add_path(PathBuf::from("/p/line/.twinova/x"))
        .add_path(PathBuf::from("/p/line/plc/Main.st"));
    assert!(relevant(folder, &renamed), "either end of a rename");
}

#[test]
fn a_write_in_place_and_a_rename_over_each_end_one_burst() {
    let (_dir, folder) = copied("two_belts_diverter");
    let (_watch, quiet) = watching(&folder);
    let started = Instant::now();
    for k in 0..5 {
        fs::write(main_st(&folder), format!("(* write {k} *)\n")).unwrap();
    }
    assert_eq!(quiet.recv_timeout(WAIT).unwrap(), Path::new(&folder));
    assert!(started.elapsed() >= QUIET, "{:?}", started.elapsed());
    assert!(quiet.recv_timeout(NOTHING).is_err(), "one burst, one read");
    // Reading the sources, as the app does once the folder is quiet, does not wake the watch again.
    assert!(changed(&project(&folder), Path::new(&folder)).is_none());
    assert!(
        quiet.recv_timeout(NOTHING).is_err(),
        "the reads are not changes"
    );
    let temporary = Path::new(&folder).join("plc/.Main.st.tmp");
    fs::write(&temporary, "(* renamed over *)\n").unwrap();
    fs::rename(&temporary, main_st(&folder)).unwrap();
    assert_eq!(quiet.recv_timeout(WAIT).unwrap(), Path::new(&folder));
}

#[test]
fn twinova_s_state_and_an_unwatched_folder_stay_quiet() {
    let (_dir, folder) = copied("two_belts_diverter");
    let (watch, quiet) = watching(&folder);
    let state = Path::new(&folder).join(".twinova/lsp");
    fs::create_dir_all(&state).unwrap();
    fs::write(state.join("field.st"), "(* generated *)\n").unwrap();
    assert!(
        quiet.recv_timeout(NOTHING).is_err(),
        ".twinova is Twinova's"
    );
    watch.follow(Some(Path::new(&folder))).unwrap();
    watch.follow(None).unwrap();
    fs::write(main_st(&folder), "(* nobody watches *)\n").unwrap();
    assert!(quiet.recv_timeout(NOTHING).is_err(), "no folder followed");
}

#[test]
fn a_folder_that_cannot_be_watched_is_an_error_naming_it() {
    let watch = SourceWatch::new(|_| {});
    let error = watch
        .follow(Some(Path::new("/no/such/folder")))
        .unwrap_err();
    assert!(error.starts_with("/no/such/folder: "), "{error}");
}

fn app() -> App<MockRuntime> {
    let app = mock_app();
    app.manage(ProjectState::default());
    app
}

/// Collects the `disk-sources` events.
fn disk_events(app: &App<MockRuntime>) -> Receiver<DiskSources> {
    #[derive(serde::Deserialize)]
    struct Payload {
        kind: String,
        program: Vec<(String, String)>,
    }
    let (tx, rx) = mpsc::channel();
    let tx = Mutex::new(tx);
    app.listen_any("disk-sources", move |e| {
        let p: Payload = serde_json::from_str(e.payload()).unwrap();
        let kind = if p.kind == "program" {
            Kind::Program
        } else {
            Kind::Behaviours
        };
        let disk = DiskSources {
            kind,
            program: p.program,
            behaviours: BTreeMap::new(),
        };
        drop(tx.lock().unwrap().send(disk));
    });
    rx
}

#[test]
fn a_quiet_folder_sends_its_changed_sources_only_while_it_is_the_project_s() {
    let app = app();
    let events = disk_events(&app);
    let (_dir, folder) = copied("two_belts_diverter");
    open_project(app.state(), folder.clone()).unwrap();
    settled(app.handle(), Path::new(&folder));
    assert!(events.recv_timeout(NOTHING).is_err(), "nothing changed");
    fs::write(main_st(&folder), "PROGRAM Main\nEND_PROGRAM\n").unwrap();
    settled(app.handle(), Path::new("/another/folder"));
    assert!(
        events.recv_timeout(NOTHING).is_err(),
        "not the project's folder"
    );
    settled(app.handle(), Path::new(&folder));
    let disk = events.recv_timeout(WAIT).unwrap();
    assert_eq!(disk.kind, Kind::Program);
    assert_eq!(disk.program[0].1, "PROGRAM Main\nEND_PROGRAM\n");
}
