use crossterm::event::KeyCode;

#[derive(Debug)]
pub enum AppAction {
    Quit,
    Help,
    RunSelected,
    OpenShell,
    Terminate,
    SetDefault,
    UnregisterPrompt,
    ShutdownPrompt,
    ExportPrompt,
    ImportPrompt,
    MoveSelection(isize),
    Ignore,
    CustomActionsPrompt,
    SearchPrompt,
    ClearSearch,
    TogglePin,
    ToggleMultiSelect,
}

pub fn map_key(code: KeyCode) -> AppAction {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => AppAction::Quit,
        KeyCode::Char('h') | KeyCode::Char('H') => AppAction::Help,
        KeyCode::Char('r') | KeyCode::Char('R') => AppAction::RunSelected,
        KeyCode::Enter => AppAction::OpenShell,
        KeyCode::Char('t') | KeyCode::Char('T') => AppAction::Terminate,
        KeyCode::Char('d') | KeyCode::Char('D') => AppAction::SetDefault,
        KeyCode::Char('u') | KeyCode::Char('U') => AppAction::UnregisterPrompt,
        KeyCode::Char('s') | KeyCode::Char('S') => AppAction::ShutdownPrompt,
        KeyCode::Char('e') | KeyCode::Char('E') => AppAction::ExportPrompt,
        KeyCode::Char('i') | KeyCode::Char('I') => AppAction::ImportPrompt,
        KeyCode::Char('a') | KeyCode::Char('A') => AppAction::CustomActionsPrompt,
        KeyCode::Char('/') => AppAction::SearchPrompt,
        KeyCode::Char('c') | KeyCode::Char('C') => AppAction::ClearSearch,
        KeyCode::Char('p') | KeyCode::Char('P') => AppAction::TogglePin,
        KeyCode::Char(' ') => AppAction::ToggleMultiSelect,
        KeyCode::Down => AppAction::MoveSelection(1),
        KeyCode::Up => AppAction::MoveSelection(-1),
        _ => AppAction::Ignore,
    }
}