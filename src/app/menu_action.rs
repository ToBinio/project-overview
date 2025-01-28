use crate::app::context_page::ContextPage;
use crate::app::Message;
use cosmic::widget::menu;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
    FocusSearch,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::OpenContextDrawer(ContextPage::About),
            MenuAction::Settings => Message::OpenContextDrawer(ContextPage::Settings),
            MenuAction::FocusSearch => Message::FocusSearchInput,
        }
    }
}
