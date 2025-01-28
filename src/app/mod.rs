use crate::app::context_page::ContextPage;
use crate::app::menu_action::MenuAction;
use crate::config::Config;
use crate::domain::program::Program;
use crate::domain::project::Project;
use crate::fl;
use cosmic::app::{context_drawer, Core, Task};
use cosmic::cosmic_config::{self};
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::iced::{event, keyboard, Event, Length, Subscription};
use cosmic::widget::menu::{Action, KeyBind};
use cosmic::widget::{self, menu};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Element, Theme};
use iter_tools::Itertools;
use log::{error, info};
use std::collections::HashMap;
use std::fs::read_dir;
use std::ops::Not;
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod context_page;
mod menu_action;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../../resources/icons/hicolor/scalable/apps/icon.svg");

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

    search_text: String,
    search_input_id: widget::Id,

    root_path_input: String,
    program_command_input: String,
    program_name_input: String,

    projects: Vec<Project>,
    programs: Vec<Program>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Key(Modifiers, Key),

    OpenContextDrawer(ContextPage),
    CloseContextDrawer,

    LaunchUrl(String),

    UpdateConfig(Config),

    RootPathInputChanged(String),
    RootPathSave(PathBuf),

    ProgramCommandInputChanged(String),
    ProgramNameInputChanged(String),
    ProgramSave,
    ProgramDelete(String),

    UpdateProjects,

    LaunchProject {
        project_name: String,
        program_name: String,
    },

    SearchTextInputChanged(String),
    FocusSearchInput,
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

        let programs = config.programs().to_vec();

        let mut key_binds = HashMap::new();

        key_binds.insert(
            KeyBind {
                modifiers: vec![],
                key: Key::Character("f".into()),
            },
            MenuAction::FocusSearch,
        );

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            key_binds: key_binds,
            // Optional configuration file for an application.
            config_handler,
            config,
            search_text: "".to_string(),
            search_input_id: widget::Id::unique(),
            root_path_input: path,
            program_command_input: "".to_string(),
            program_name_input: "".to_string(),
            projects: vec![],
            programs,
        };

        info!("{:?}", app.config.project_root_path());

        let update_title_task = app.update_title();
        let task = Task::batch(vec![
            update_title_task,
            Task::done(cosmic::app::Message::App(Message::UpdateProjects)),
            Task::done(cosmic::app::Message::App(Message::FocusSearchInput)),
        ]);

        (app, task)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(self.context_page.view(self))
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
        let subscriptions = vec![
            event::listen_with(|event, status, window_id| match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match status
                {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                _ => None,
            }),
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ];

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
            }
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
                    error!("failed to open {url:?}: {err}");
                }
            },
            Message::LaunchProject {
                program_name,
                project_name,
            } => {
                let Some(program) = self
                    .programs
                    .iter()
                    .find(|program| program.name() == program_name)
                else {
                    return Task::none();
                };

                let mut path = self.config.project_root_path().unwrap().clone();
                path.push(project_name);

                let command = program.command().replace("%path%", path.to_str().unwrap());
                let mut command = command.split_whitespace();

                let exec = command.next().unwrap();
                let args: Vec<&str> = command.collect();

                let _ = Command::new(exec)
                    .args(args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            }
            Message::RootPathInputChanged(path) => {
                self.root_path_input = path;
            }
            Message::RootPathSave(path) => {
                info!("saving root path - {:?}", path);
                let _ = self
                    .config
                    .set_project_root_path(self.config_handler.as_ref().unwrap(), Some(path));
            }
            Message::ProgramCommandInputChanged(cmd) => {
                self.program_command_input = cmd;
            }
            Message::ProgramNameInputChanged(name) => {
                self.program_name_input = name;
            }
            Message::ProgramSave => {
                let program = Program::new(
                    self.program_name_input.clone(),
                    self.program_command_input.clone(),
                );
                info!("saving program - {:?}", program);

                self.programs.push(program);
                self.program_command_input = "".to_string();
                self.program_name_input = "".to_string();

                self.save_programs();
            }
            Message::ProgramDelete(name) => {
                self.programs.retain(|program| program.name() != name);
                self.save_programs();
            }
            Message::UpdateProjects => {
                let Some(path) = self.config.project_root_path() else {
                    return Task::none();
                };

                let result = read_dir(path).unwrap();

                self.projects = result
                    .filter_map(|dir| dir.ok())
                    .filter_map(|dir| match dir.try_into() {
                        Ok(project) => Some(project),
                        Err(err) => {
                            error!("{}", err);
                            None
                        }
                    })
                    .collect();
            }
            Message::SearchTextInputChanged(text) => {
                self.search_text = text;
            }
            Message::FocusSearchInput => {
                return widget::text_input::focus(self.search_input_id.clone())
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let theme = theme::active();
        let cosmic_theme::Spacing { space_xs, .. } = theme.cosmic().spacing;

        let input = widget::search_input(fl!("search-input"), &self.search_text)
            .on_input(Message::SearchTextInputChanged)
            .id(self.search_input_id.clone());

        widget::Column::new()
            .push(input)
            .push(self.projects(&theme))
            .spacing(space_xs)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl AppModel {
    fn filter_projects(&self) -> Vec<&Project> {
        self.projects
            .iter()
            .filter(|project| {
                self.search_text.is_empty() || project.name().contains(&self.search_text)
            })
            .sorted_by(|a, b| b.modify().cmp(a.modify()))
            .collect()
    }

    fn projects(&self, theme: &Theme) -> Element<Message> {
        let cosmic_theme::Spacing { space_xs, .. } = theme.cosmic().spacing;

        let mut column = widget::Column::new();

        for project in &self.filter_projects() {
            column = column.push(self.project(project));
        }

        widget::scrollable(column)
            .spacing(space_xs)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    fn project(&self, project: &Project) -> Element<Message> {
        let mut programs = widget::Row::new();

        for program in &self.programs {
            programs = programs.push(widget::button::text(program.name()).on_press(
                Message::LaunchProject {
                    program_name: program.name().to_string(),
                    project_name: project.name().to_string(),
                },
            ));
        }

        widget::Column::new()
            .push(widget::text::text(project.name().to_string()))
            .push(programs)
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

    fn save_programs(&mut self) {
        let _ = self.config.set_programs(
            self.config_handler.as_ref().unwrap(),
            self.programs.to_vec(),
        );
    }

    pub fn is_valid_program(&self) -> bool {
        let name = &self.program_name_input;
        let command = &self.program_command_input;

        name.is_empty().not()
            && Program::is_valid_command(command)
            && self.programs.iter().any(|p| p.name() == name).not()
    }
}
