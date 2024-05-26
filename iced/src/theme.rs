use iced::{Application, application, Color, Element, Executor, executor, Row};
use crate::{Command, Length, Renderer};
use iced::Container;

#[derive(Clone, Copy, Default)]
pub struct ThemeApp {}

impl Application for ThemeApp {
    type Executor = executor::Default;
    type Message = ();
    type Theme = theme::Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self{ }, Command::none() )
    }

    fn title(&self) -> String {
        "Title".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Container::new(
            Container::new(Row::new())
                .width(Length::Units(50))
                .height(Length::Units(50))
                .style(theme::Container::Bordered),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

impl application::StyleSheet for theme::Theme {
    type Style = ThemeApp;

    fn appearance(&self, style: Self::Style) -> application::Appearance {
        // let palette = self.extended_palette();

        application::Appearance {
            background_color: Color::from_rgb(1.0, 0., 0.),
            text_color: Color::from_rgb(0., 1.0, 0.),
        }
    }
}


pub mod theme {
    use iced::application;
    use iced::widget::container;
    use iced::Color;

    #[derive(Default)]
    pub struct Theme {}

    #[derive(Default, Clone, Copy)]
    pub enum Container {
        #[default]
        Default,
        Bordered,
    }

    impl container::StyleSheet for Theme {
        type Style = Container;

        fn appearance(&self, style: Self::Style) -> container::Appearance {
            match style {
                Container::Default => container::Appearance::default(),
                Container::Bordered => container::Appearance {
                    border_width: 1.0,
                    border_color: Color::BLACK,
                    ..Default::default()
                },
            }
        }
    }
}