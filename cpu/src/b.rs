use std::sync::mpsc;
use iced::{Application, Command, Element, executor, Subscription};
use std::sync::mpsc::{Receiver};
use iced_futures::subscription::Recipe;
use futures::prelude::stream::BoxStream;
use crate::CpuMessage;

struct Emulator {
    rx: Receiver<CpuMessage>,
}

impl<H, I> Recipe<H, I> for Emulator
    where H: std::hash::Hasher
{
    type Output = (CpuMessage, ());

    fn hash(&self, state: &mut H) {
        // struct Marker;
        // std::any::TypeId::of::<Marker>().hash(state);
        //
        // self.id.hash(state);
    }

    fn stream(self: Box<Self>, input: BoxStream<I>) -> BoxStream<Self::Output> {
        use futures::stream::{self, StreamExt};
        use futures::future;

        let stream = futures::stream::unfold(self.rx, |r| {
            let v = match r.recv() {
                Ok(v) => {
                    println!("Received value from rx: {:?}", v);
                    // Some((v, r))
                    Some(((v, ()), r))
                },
                Err(_) => {
                    println!("StreamGenerator in error");
                    None
                },
            };
            future::ready(v)
        });
        Box::pin(stream)
    }
}

impl Application for Emulator {
    type Executor = executor::Default;
    type Message = CpuMessage;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (tx, rx) = mpsc::channel::<CpuMessage>();

        let result = Emulator {
            rx,
        };

        (result, Command::none())
    }

    fn title(&self) -> String { todo!() }
    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> { todo!() }

    // fn subscription(&self) -> Subscription<CpuMessage> {
    //     iced::Subscription::from_recipe(self)    // error here
    // }

    fn view(&mut self) -> Element<'_, Self::Message> { todo!() }

}

