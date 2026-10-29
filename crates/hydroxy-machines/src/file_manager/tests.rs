use super::*;
use hydroxy_statechart::{Machine, step as advance};

fn items() -> Vec<FileItem> {
    vec![
        FileItem::folder("docs", "Docs", ROOT),
        FileItem::folder("img", "images", ROOT),
        FileItem::file("a", "b-notes.md", ROOT, 10),
        FileItem::file("b", "A-readme.md", ROOT, 20),
        FileItem::folder("old", "Old", "docs"),
        FileItem::file("c", "spec.pdf", "docs", 30),
    ]
}

fn input() -> FileManagerInput {
    FileManagerInput { items: items(), ..FileManagerInput::default() }
}

fn click(id: &str, extend: Extend) -> FileManagerEvent {
    FileManagerEvent::Click { id: id.into(), extend }
}

fn ids(v: &[FileItem]) -> Vec<&str> {
    v.iter().map(|i| i.id.as_str()).collect()
}

#[test]
fn folders_come_first_then_names_ignoring_case() {
    assert_eq!(ids(&visible(&items(), ROOT)), ["docs", "img", "b", "a"]);
    assert_eq!(ids(&visible(&items(), "docs")), ["old", "c"]);
    assert_eq!(ids(&path(&items(), "old")), ["docs", "old"]);
    assert!(path(&items(), ROOT).is_empty());
}

#[test]
fn open_up_and_activate_move_between_folders() {
    let i = input();
    let s = FileManagerMachine::initial(&i);
    let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Activate("docs".into()), &i);
    assert_eq!(step.state.folder.value, "docs");
    assert_eq!(step.commands, [FileManagerCommand::FolderChange("docs".into())]);
    let step = advance::<FileManagerMachine>(&step.state, FileManagerEvent::Up, &i);
    assert_eq!(step.state.folder.value, ROOT);
    assert!(advance::<FileManagerMachine>(&step.state, FileManagerEvent::Up, &i).commands.is_empty());
    assert!(advance::<FileManagerMachine>(&s, FileManagerEvent::Open("a".into()), &i).commands.is_empty());
    let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Activate("a".into()), &i);
    assert_eq!(step.commands, [FileManagerCommand::OpenFile("a".into())]);
}

#[test]
fn leaving_a_folder_clears_the_selection() {
    let i = input();
    let s = advance::<FileManagerMachine>(&FileManagerMachine::initial(&i), click("a", Extend::Replace), &i).state;
    let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Open("docs".into()), &i);
    assert!(step.state.selection.is_empty());
    assert_eq!(step.commands[1], FileManagerCommand::SelectionChange(Vec::new()));
}

#[test]
fn click_replaces_toggles_and_ranges() {
    let i = input();
    let s = FileManagerMachine::initial(&i);
    let s = advance::<FileManagerMachine>(&s, click("img", Extend::Replace), &i).state;
    let s = advance::<FileManagerMachine>(&s, click("a", Extend::Range), &i).state;
    assert_eq!(s.selection, ["img", "b", "a"]);
    let s = advance::<FileManagerMachine>(&s, click("b", Extend::Toggle), &i).state;
    assert_eq!(s.selection, ["img", "a"]);
    let step = advance::<FileManagerMachine>(&s, click("docs", Extend::Replace), &i);
    assert_eq!(step.state.selection, ["docs"]);
    assert_eq!(step.commands, [FileManagerCommand::SelectionChange(vec!["docs".into()])]);
    // Clicking what is already the whole selection reports nothing.
    assert!(advance::<FileManagerMachine>(&step.state, click("docs", Extend::Replace), &i).commands.is_empty());
}

#[test]
fn arrows_move_focus_and_selection() {
    let i = input();
    let s = FileManagerMachine::initial(&i);
    let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Move { step: Step::By(1), extend: false }, &i);
    assert_eq!(step.state.focused.as_deref(), Some("docs"));
    assert_eq!(step.state.selection, ["docs"]);
    let s = advance::<FileManagerMachine>(&step.state, FileManagerEvent::Move { step: Step::By(2), extend: true }, &i)
        .state;
    assert_eq!(s.focused.as_deref(), Some("b"));
    assert_eq!(s.selection, ["docs", "img", "b"]);
    let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Move { step: Step::Last, extend: false }, &i);
    assert_eq!(step.state.focused.as_deref(), Some("a"));
    assert_eq!(step.commands.last(), Some(&FileManagerCommand::FocusItem("a".into())));
    // At the end already: nothing happens.
    assert!(
        advance::<FileManagerMachine>(&step.state, FileManagerEvent::Move { step: Step::By(5), extend: false }, &i)
            .commands
            .is_empty()
    );
}

#[test]
fn context_keeps_a_selection_it_is_inside() {
    let i = input();
    let s = advance::<FileManagerMachine>(&FileManagerMachine::initial(&i), click("docs", Extend::Replace), &i).state;
    let s = advance::<FileManagerMachine>(&s, click("a", Extend::Range), &i).state;
    assert!(advance::<FileManagerMachine>(&s, FileManagerEvent::Context("img".into()), &i).commands.is_empty());
    let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Context("img".into()), &i);
    assert_eq!(step.state.selection, ["docs", "img", "b", "a"]);
    let step =
        advance::<FileManagerMachine>(&FileManagerMachine::initial(&i), FileManagerEvent::Context("a".into()), &i);
    assert_eq!(step.state.selection, ["a"]);
}

#[test]
fn rename_validates_and_reports_a_change() {
    let i = input();
    let s = FileManagerMachine::initial(&i);
    let s = advance::<FileManagerMachine>(&s, FileManagerEvent::StartRename("a".into()), &i).state;
    assert_eq!(s.renaming.as_ref().map(|r| r.draft.as_str()), Some("b-notes.md"));
    let blank = advance::<FileManagerMachine>(
        &advance::<FileManagerMachine>(&s, FileManagerEvent::RenameInput("  ".into()), &i).state,
        FileManagerEvent::CommitRename,
        &i,
    );
    assert_eq!(blank.state.renaming.as_ref().and_then(|r| r.error), Some(NameError::Empty));
    let taken = advance::<FileManagerMachine>(
        &advance::<FileManagerMachine>(&s, FileManagerEvent::RenameInput("A-README.MD".into()), &i).state,
        FileManagerEvent::CommitRename,
        &i,
    );
    assert_eq!(taken.state.renaming.as_ref().and_then(|r| r.error), Some(NameError::Taken));
    // Typing clears the error.
    assert_eq!(
        advance::<FileManagerMachine>(&taken.state, FileManagerEvent::RenameInput("x".into()), &i)
            .state
            .renaming
            .and_then(|r| r.error),
        None
    );
    let ok = advance::<FileManagerMachine>(
        &advance::<FileManagerMachine>(&s, FileManagerEvent::RenameInput(" notes.md ".into()), &i).state,
        FileManagerEvent::CommitRename,
        &i,
    );
    assert!(ok.state.renaming.is_none());
    assert_eq!(ok.commands[0], FileManagerCommand::Rename { id: "a".into(), name: "notes.md".into() });
    // The same name just closes.
    let same = advance::<FileManagerMachine>(&s, FileManagerEvent::CommitRename, &i);
    assert!(!same.commands.iter().any(|c| matches!(c, FileManagerCommand::Rename { .. })));
    let cancelled = advance::<FileManagerMachine>(&s, FileManagerEvent::CancelRename, &i);
    assert!(cancelled.state.renaming.is_none());
}

#[test]
fn delete_asks_the_owner_and_sync_prunes() {
    let mut i = input();
    let s = advance::<FileManagerMachine>(&FileManagerMachine::initial(&i), click("docs", Extend::Replace), &i).state;
    let s = advance::<FileManagerMachine>(&s, click("a", Extend::Toggle), &i).state;
    let step = advance::<FileManagerMachine>(&s, FileManagerEvent::DeleteSelected, &i);
    assert_eq!(step.commands, [FileManagerCommand::Delete(vec!["docs".into(), "a".into()])]);
    assert_eq!(step.state.selection.len(), 2);
    apply_delete(&mut i.items, &["docs".to_string(), "a".to_string()]);
    assert_eq!(ids(&visible(&i.items, ROOT)), ["img", "b"]);
    assert!(i.items.iter().all(|x| x.id != "c" && x.id != "old"));
    assert!(hydroxy_statechart::sync_or_keep::<FileManagerMachine>(&step.state, &i).selection.is_empty());
}

#[test]
fn controlled_folder_waits_for_the_owner() {
    let mut i = FileManagerInput { folder: Some(ROOT.into()), ..input() };
    let s = FileManagerMachine::initial(&i);
    let step = advance::<FileManagerMachine>(&s, FileManagerEvent::Open("docs".into()), &i);
    assert_eq!(step.state.folder.value, ROOT);
    assert_eq!(step.commands, [FileManagerCommand::FolderChange("docs".into())]);
    i.folder = Some("docs".into());
    assert_eq!(hydroxy_statechart::sync_or_keep::<FileManagerMachine>(&step.state, &i).folder.value, "docs");
    // A folder that no longer exists falls back to the top.
    i.folder = Some("gone".into());
    assert_eq!(hydroxy_statechart::sync_or_keep::<FileManagerMachine>(&step.state, &i).folder.value, ROOT);
}

#[test]
fn apply_rename_and_view() {
    let mut list = items();
    apply_rename(&mut list, "a", "z.md");
    assert_eq!(list[2].name, "z.md");
    let i = input();
    let s = FileManagerMachine::initial(&i);
    assert_eq!(
        advance::<FileManagerMachine>(&s, FileManagerEvent::SetView(ViewMode::Grid), &i).state.view,
        ViewMode::Grid
    );
    assert!(advance::<FileManagerMachine>(&s, FileManagerEvent::SetView(ViewMode::List), &i).commands.is_empty());
}
