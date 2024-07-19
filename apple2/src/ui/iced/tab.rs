use iced::*;
use iced::alignment::*;
use iced::widget::*;
use iced_aw::TabLabel;
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::style::m_container;

const HEADER_SIZE: u16 = 32;
const TAB_PADDING: u16 = 0;

pub trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
    }

    fn view(&self) -> Element<'_, InternalUiMessage> {
        let column = Column::new()
            // .spacing(20)
            .push(self.content())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(iced::Alignment::Center)
            ;

        Container::new(m_container(column.into()))
            // .width(Length::Fill)
            // .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&self) -> Element<'_, InternalUiMessage>;
}
