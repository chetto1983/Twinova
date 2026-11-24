use super::*;
use serde_json::json;
use tauri::Listener;
use tauri::test::{MockRuntime, mock_app};

/// A mock app whose window's registry listens.
fn app() -> tauri::App<MockRuntime> {
    let app = mock_app();
    app.manage(Control::default());
    control_ready(app.state());
    app
}

/// A request reaches the window as an event with its id, and the window's answer, through `control_reply`, is what the
/// client gets: a value, or an error's text.
#[tokio::test]
async fn a_request_reaches_the_window_and_its_reply_comes_back() {
    let app = app();
    let handle = app.handle().clone();
    let seen = std::sync::Arc::new(Mutex::new(Vec::<Value>::new()));
    let kept = std::sync::Arc::clone(&seen);
    app.listen_any("control-request", move |event| {
        kept.lock()
            .unwrap()
            .push(serde_json::from_str(event.payload()).unwrap());
    });
    let asking = {
        let handle = handle.clone();
        tokio::spawn(async move {
            let control = handle.state::<Control>();
            let first = control
                .ask(&handle, "build".into(), json!({ "a": 1 }))
                .await;
            let second = control.ask(&handle, "step".into(), json!(null)).await;
            (first, second)
        })
    };
    let answered = |id: u64, ok: bool, value: Value| {
        let handle = handle.clone();
        async move {
            while !control_reply(handle.state(), id, ok, value.clone()) {
                tokio::task::yield_now().await;
            }
        }
    };
    answered(1, true, json!({ "built": true })).await;
    answered(2, false, json!("build the project first")).await;
    let (first, second) = asking.await.unwrap();
    assert_eq!(first, Ok(json!({ "built": true })));
    assert_eq!(second, Err("build the project first".to_string()));
    assert_eq!(
        *seen.lock().unwrap(),
        [
            json!({ "id": 1, "method": "build", "params": { "a": 1 } }),
            json!({ "id": 2, "method": "step", "params": null })
        ]
    );
    // an error that is no string is answered as its JSON, and an id nothing waits for any more is said so
    assert!(!control_reply(handle.state(), 2, true, json!(1)));
}

#[tokio::test(start_paused = true)]
async fn a_window_that_does_not_answer_in_time_is_said_so_and_its_late_answer_dropped() {
    let app = app();
    let handle = app.handle().clone();
    let answer = handle
        .state::<Control>()
        .ask(&handle, "build".into(), json!({}))
        .await;
    assert_eq!(
        answer,
        Err("the window did not answer build within 60 s".to_string())
    );
    assert!(!control_reply(handle.state(), 1, true, json!(true)));
}

#[tokio::test]
async fn an_error_that_is_no_text_comes_back_as_its_json() {
    let app = app();
    let handle = app.handle().clone();
    let asking = {
        let handle = handle.clone();
        tokio::spawn(async move {
            handle
                .state::<Control>()
                .ask(&handle, "x".into(), json!({}))
                .await
        })
    };
    while !control_reply(handle.state(), 1, false, json!({ "code": 3 })) {
        tokio::task::yield_now().await;
    }
    assert_eq!(asking.await.unwrap(), Err(r#"{"code":3}"#.to_string()));
}

#[test]
fn the_channel_is_turned_off_by_the_variable_or_by_a_file() {
    let data = tempfile::tempdir().unwrap();
    assert!(!turned_off(None, data.path()));
    assert!(!turned_off(Some("on".into()), data.path()));
    assert!(turned_off(Some("OFF".into()), data.path()));
    std::fs::create_dir_all(data.path().join("control")).unwrap();
    std::fs::write(data.path().join("control/off"), "").unwrap();
    assert!(turned_off(None, data.path()));
}

/// The channel opened for the app: the port the status bar asks for, the file written for its user, a client with its
/// token answered by the window, and the file removed when the app quits.
#[tokio::test]
async fn the_channel_opens_writes_its_file_and_removes_it_at_exit() {
    let app = app();
    let handle = app.handle().clone();
    let data = tempfile::tempdir().unwrap();
    assert_eq!(control_port(handle.state()), None);
    let opening = {
        let (handle, data) = (handle.clone(), data.path().to_path_buf());
        tokio::spawn(async move { open(&handle, data).await })
    };
    let file = loop {
        if let Ok(file) = ControlFile::read(data.path()) {
            break file;
        }
        tokio::task::yield_now().await;
    };
    while control_port(handle.state()).is_none() {
        tokio::task::yield_now().await;
    }
    assert_eq!(control_port(handle.state()), Some(file.port));
    assert!(PORTS.contains(&file.port));
    let window = handle.clone();
    app.listen_any("control-request", move |event| {
        let request: Value = serde_json::from_str(event.payload()).unwrap();
        let id = request["id"].as_u64().unwrap();
        control_reply(window.state(), id, true, request["params"].clone());
    });
    let mut client = twinova_control::Client::connect(file.port, &file.token)
        .await
        .unwrap();
    assert_eq!(client.call("echo", json!([1, 2])).await, Ok(json!([1, 2])));
    close(&handle);
    assert!(ControlFile::read(data.path()).is_err());
    assert_eq!(control_port(handle.state()), None);
    opening.abort();
}

/// A request sent before the window's registry listens waits for it, and one the window never gets ready for is said so.
#[tokio::test(start_paused = true)]
async fn a_request_waits_for_the_window_to_be_ready() {
    let app = mock_app();
    app.manage(Control::default());
    let handle = app.handle().clone();
    let waiting = {
        let handle = handle.clone();
        tokio::spawn(async move {
            let control = handle.state::<Control>();
            control.ask(&handle, "state".into(), json!({})).await
        })
    };
    tokio::task::yield_now().await;
    assert!(
        !control_reply(handle.state(), 1, true, json!(true)),
        "asked before ready"
    );
    control_ready(handle.state());
    while !control_reply(handle.state(), 1, true, json!("ready")) {
        tokio::task::yield_now().await;
    }
    assert_eq!(waiting.await.unwrap(), Ok(json!("ready")));

    let never = mock_app();
    never.manage(Control::default());
    let handle = never.handle().clone();
    let control = handle.state::<Control>();
    let answer = control.ask(&handle, "state".into(), json!({})).await;
    assert_eq!(
        answer,
        Err("the window was not ready for state within 60 s".to_string())
    );
}

/// The agent finds the app's file by the identifier it shares with tauri.conf.json, which names Tauri's data folder.
#[test]
fn the_agent_looks_in_the_folder_tauri_gives_the_app() {
    let conf: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert_eq!(conf["identifier"], json!(twinova_control::file::IDENTIFIER));
    let data = twinova_control::file::app_data_dir().unwrap();
    assert!(
        data.ends_with(twinova_control::file::IDENTIFIER),
        "{}",
        data.display()
    );
}

fn listed(names: &[&str]) -> Vec<Listed> {
    (names.iter())
        .map(|n| Listed {
            name: (*n).to_string(),
            description: format!("{n}'s"),
        })
        .collect()
}

/// Each window's registry tells its commands; a request goes to the window holding its command, the PLC window on a tie,
/// the main window when none holds it; `list_commands` merges them; a window gone takes its commands away.
#[tokio::test]
async fn a_request_goes_to_the_window_that_holds_its_command() {
    let app = app();
    let handle = app.handle().clone();
    let control = handle.state::<Control>();
    control.set_commands("main", listed(&["build", "values"]));
    control.set_commands("plc", listed(&["values", "breakpoints"]));
    let routes =
        || ["build", "values", "breakpoints", "fly"].map(|name| control.window_of(&handle, name));
    assert_eq!(
        routes(),
        ["main"; 4],
        "no PLC window open: its commands go nowhere"
    );
    let names = |v: Value| -> Vec<String> {
        (v.as_array().unwrap().iter())
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect()
    };
    let merged = control
        .ask(&handle, "list_commands".into(), json!({}))
        .await
        .unwrap();
    assert_eq!(names(merged), ["breakpoints", "build", "values"]);
    // the request is emitted to the PLC window only
    let seen = std::sync::Arc::new(Mutex::new(Vec::<String>::new()));
    for label in ["main", "plc"] {
        let kept = std::sync::Arc::clone(&seen);
        app.listen_any(format!("seen-{label}"), move |_| {
            kept.lock().unwrap().push(label.into())
        });
    }
    let to = |label: &'static str| {
        let handle = handle.clone();
        move |_: tauri::Event| {
            let _ = handle.emit(&format!("seen-{label}"), ());
        }
    };
    let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let plc = tauri::WebviewWindowBuilder::new(&app, "plc", Default::default())
        .build()
        .unwrap();
    main.listen("control-request", to("main"));
    plc.listen("control-request", to("plc"));
    assert_eq!(
        routes(),
        ["main", "plc", "plc", "main"],
        "the PLC window wins a tie, and a command nobody holds goes to the main window, which refuses it"
    );
    let asking = {
        let handle = handle.clone();
        tokio::spawn(async move {
            (handle.state::<Control>())
                .ask(
                    &handle,
                    "describe_command".into(),
                    json!({ "name": "breakpoints" }),
                )
                .await
        })
    };
    while !control_reply(handle.state(), 1, true, json!("described")) {
        tokio::task::yield_now().await;
    }
    assert_eq!(asking.await.unwrap(), Ok(json!("described")));
    assert_eq!(*seen.lock().unwrap(), ["plc"]);
    control.set_commands("plc", Vec::new());
    assert_eq!(control.window_of(&handle, "breakpoints"), "main");
    assert_eq!(
        names(
            control
                .ask(&handle, "list_commands".into(), json!({}))
                .await
                .unwrap()
        ),
        ["build", "values"]
    );
}

/// A PLC window just opened has not told its commands yet: a request for one of the pane's waits for it, rather than
/// going to the main window, which gave the pane up.
#[tokio::test(start_paused = true)]
async fn a_request_waits_for_a_plc_window_that_is_loading() {
    let app = app();
    let handle = app.handle().clone();
    handle
        .state::<Control>()
        .set_commands("main", listed(&["build"]));
    let seen = std::sync::Arc::new(Mutex::new(Vec::<String>::new()));
    for label in ["main", "plc"] {
        let window = tauri::WebviewWindowBuilder::new(&app, label, Default::default())
            .build()
            .unwrap();
        let kept = std::sync::Arc::clone(&seen);
        window.listen("control-request", move |_| {
            kept.lock().unwrap().push(label.into())
        });
    }
    let asking = {
        let handle = handle.clone();
        tokio::spawn(async move {
            (handle.state::<Control>())
                .ask(&handle, "values".into(), json!({}))
                .await
        })
    };
    tokio::task::yield_now().await;
    assert!(
        seen.lock().unwrap().is_empty(),
        "nothing sent while the PLC window loads"
    );
    handle
        .state::<Control>()
        .set_commands("plc", listed(&["values"]));
    // the clock is paused: a sleep lets the request's own wait go by
    while !control_reply(handle.state(), 1, true, json!([])) {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(asking.await.unwrap(), Ok(json!([])));
    assert_eq!(*seen.lock().unwrap(), ["plc"]);
    // a command no window holds, the PLC window loaded, goes to the main window at once
    let fly = {
        let handle = handle.clone();
        tokio::spawn(async move {
            (handle.state::<Control>())
                .ask(&handle, "fly".into(), json!({}))
                .await
        })
    };
    while !control_reply(handle.state(), 2, false, json!("no command fly")) {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(fly.await.unwrap(), Err("no command fly".into()));
    assert_eq!(seen.lock().unwrap().last().unwrap(), "main");
}

/// `control_commands`, as a window's registry invokes it, keeps the commands under that window's own label.
#[tokio::test]
async fn a_window_tells_its_commands_under_its_own_label() {
    let app = app();
    let handle = app.handle().clone();
    let plc = tauri::WebviewWindowBuilder::new(&app, "plc", Default::default())
        .build()
        .unwrap();
    control_commands(plc.as_ref().window(), app.state(), listed(&["values"]));
    assert_eq!(
        handle.state::<Control>().window_of(&handle, "values"),
        "plc"
    );
}

/// While a PLC window loads, only a command nobody holds waits for it: the main window's own commands go at once, and,
/// the PLC window loaded, a command nobody holds goes at once to the main window, which refuses it.
#[tokio::test(start_paused = true)]
async fn only_the_pane_s_commands_wait_for_a_loading_plc_window() {
    let app = app();
    let handle = app.handle().clone();
    let control = handle.state::<Control>();
    control.set_commands("main", listed(&["build"]));
    tauri::WebviewWindowBuilder::new(&app, "plc", Default::default())
        .build()
        .unwrap();
    let at_once = |name: &'static str| {
        let handle = handle.clone();
        async move {
            let control = handle.state::<Control>();
            tokio::time::timeout(Duration::from_millis(1), control.window_for(&handle, name)).await
        }
    };
    assert_eq!(at_once("build").await, Ok("main".to_string()));
    assert!(at_once("values").await.is_err(), "the pane's command waits");
    control.set_commands("plc", listed(&["values"]));
    assert_eq!(at_once("values").await, Ok("plc".to_string()));
    assert_eq!(at_once("fly").await, Ok("main".to_string()));
}
