use iced::widget::{button, center, text, Column, Row};
use iced::{Alignment, Element};

#[derive(Default)]
pub struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum CounterMessage {
    Increment,
    Decrement,
}

impl Counter {
    fn new() -> Counter {
        Counter { value: 0}
    }
    fn update(&mut self, message: CounterMessage) {
        match message {
            CounterMessage::Increment => self.value += 1,
            CounterMessage::Decrement => self.value -= 1,
        }
    }
    fn view(&self) -> Element<'_, CounterMessage> {
        let increment = button("+").on_press(CounterMessage::Increment);
        let decrement = button("-").on_press(CounterMessage::Decrement);
        let counter_view = text(self.value.to_string()).size(50);

        Row::new()
            .push(decrement)
            .push(counter_view)
            .push(increment)
            .spacing(20)
            .align_y(Alignment::Center)
            .into()
    }
}

#[derive(Default)]
struct App {
    counter1: Counter,
    counter2: Counter,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Counter1(CounterMessage),
    Counter2(CounterMessage),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Counter1(msg) => self.counter1.update(msg),
            Message::Counter2(msg) => self.counter2.update(msg),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        center(Column::new()
            .push(self.counter1.view().map(Message::Counter1))
            .push(self.counter2.view().map(Message::Counter2))
            .spacing(20)
            .align_x(Alignment::Center)).into()
    }
}

fn main() -> iced::Result{
    iced::run(App::update, App::view)
}

#[test]
fn it_counts_properly() {
    let mut counter = Counter::new();

    counter.update(CounterMessage::Increment);
    counter.update(CounterMessage::Increment);
    counter.update(CounterMessage::Decrement);

    assert_eq!(counter.value, 1);
}