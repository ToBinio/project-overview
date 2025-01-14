use crate::config::Config;
use crate::fl;
use cosmic::app::{context_drawer, Core, Task};
use cosmic::cosmic_config::{self};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::widget::{self, menu};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Apply, Element};
use std::collections::HashMap;
use std::path::PathBuf;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    // Configuration data that persists between application runs.
    config_handler: Option<cosmic_config::Config>,
    config: Config,

    root_path_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenContextDrawer(ContextPage),
    CloseContextDrawer,

    LaunchUrl(String),

    UpdateConfig(Config),

    RootPathInputChanged(String),
    RootPathSave(PathBuf),
}

impl Application for AppModel {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "at.tobinio.ProjectOverview";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let (config_handler, config) = Config::load();

        let path = config
            .project_root_path()
            .map(|path| path.to_str().unwrap_or_default())
            .unwrap_or_default()
            .to_string();

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config_handler,
            config,
            root_path_input: path,
        };

        println!("{:?}", app.config.project_root_path());

        let command = app.update_title();

        (app, command)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => {
                context_drawer::context_drawer(self.about(), Message::CloseContextDrawer)
                    .title(fl!("about"))
            }
            ContextPage::Settings => {
                context_drawer::context_drawer(self.settings(), Message::CloseContextDrawer)
                    .title(fl!("settings"))
            }
        })
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                    menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::OpenContextDrawer(context_page) => {
                self.context_page = context_page;
                self.core.window.show_context = true;
            }
            Message::CloseContextDrawer => {
                self.core.window.show_context = false;
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::RootPathInputChanged(path) => {
                self.root_path_input = path;
            }
            Message::RootPathSave(path) => {
                println!("saving root path - {:?}", path);
                let _ = self
                    .config
                    .set_project_root_path(self.config_handler.as_ref().unwrap(), Some(path));
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        widget::text::title1(fl!("welcome"))
            .apply(widget::container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}

impl AppModel {
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));

        let title = widget::text::title3(fl!("app-title"));

        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
                .padding(0),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn settings(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let path_buf = PathBuf::from(&self.root_path_input);

        let input = widget::text_input(fl!("settings-path-placeholder"), &self.root_path_input)
            .on_input(Message::RootPathInputChanged);

        let mut save = widget::button::text(fl!("save"));

        if path_buf.exists() {
            save = save.on_press(Message::RootPathSave(path_buf));
        }

        widget::column()
            .push(input)
            .push(save)
            .spacing(space_xxs)
            .into()
    }

    pub fn update_title(&mut self) -> Task<Message> {
        let window_title = fl!("app-title");

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::OpenContextDrawer(ContextPage::About),
            MenuAction::Settings => Message::OpenContextDrawer(ContextPage::Settings),
        }
    }
}
