use crate::app::{AppModel, Message, APP_ICON, REPOSITORY};
use crate::domain::program::Program;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::iced::Alignment;
use cosmic::iced_core::Theme;
use cosmic::widget::icon::Handle;
use cosmic::{cosmic_theme, theme, widget, Element};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

impl ContextPage {
    pub fn view<'a>(&self, app: &'a AppModel) -> context_drawer::ContextDrawer<'a, Message> {
        match self {
            ContextPage::About => {
                context_drawer::context_drawer(Self::about(app), Message::CloseContextDrawer)
                    .title(fl!("about"))
            }
            ContextPage::Settings => {
                context_drawer::context_drawer(Self::settings(app), Message::CloseContextDrawer)
                    .title(fl!("settings"))
            }
        }
    }
    fn about(_app: &AppModel) -> Element<Message> {
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

    fn settings(app: &AppModel) -> Element<Message> {
        let theme = theme::active();
        let cosmic_theme::Spacing { space_xs, .. } = theme.cosmic().spacing;

        widget::column()
            .push(Self::root_path(app, &theme))
            .push(widget::divider::horizontal::default())
            .push(Self::program_input(app, &theme))
            .push(Self::programs(app, &theme))
            .spacing(space_xs)
            .into()
    }

    fn root_path<'a>(app: &'a AppModel, theme: &cosmic::Theme) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme.cosmic().spacing;

        let path_buf = PathBuf::from(&app.root_path_input);

        let input = widget::text_input(fl!("settings-path-placeholder"), &app.root_path_input)
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

    fn program_input<'a>(app: &'a AppModel, theme: &cosmic::Theme) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme.cosmic().spacing;

        let command_input = widget::text_input(
            fl!("settings-program-command-placeholder"),
            &app.program_command_input,
        )
        .on_input(Message::ProgramCommandInputChanged);

        let name_input = widget::text_input(
            fl!("settings-program-name-placeholder"),
            &app.program_name_input,
        )
        .on_input(Message::ProgramNameInputChanged);
        let mut add = widget::button::text(fl!("add"));

        if app.is_valid_program() {
            add = add.on_press(Message::ProgramSave);
        }

        widget::column()
            .push(command_input)
            .push(name_input)
            .push(add)
            .spacing(space_xxs)
            .into()
    }

    fn programs<'a>(app: &'a AppModel, theme: &cosmic::Theme) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme.cosmic().spacing;

        let mut column = widget::column().spacing(space_xxs);

        for program in &app.programs {
            column = column.push(widget::divider::horizontal::light());
            column = column.push(Self::program(app, theme, program));
        }

        column.into()
    }

    fn program<'a>(
        _app: &'a AppModel,
        theme: &cosmic::Theme,
        program: &'a Program,
    ) -> Element<'a, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme.cosmic().spacing;

        let name = widget::text::text(program.name());
        let command = widget::text::caption(program.command());

        let column = widget::column().push(name).push(command);

        let delete_button = widget::button::icon(widget::icon::from_name("edit-delete-symbolic"))
            .on_press(Message::ProgramDelete(program.name().to_string()));

        widget::row()
            .spacing(space_xxs)
            .push(column)
            .push(delete_button)
            .into()
    }
}
