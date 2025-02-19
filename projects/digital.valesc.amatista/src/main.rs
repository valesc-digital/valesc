// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

#[derive(Default)]
struct Counter {
    value: u64,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

use iced::widget::{button, text};
use iced::Element;

fn update(counter: &mut Counter, message: Message) {
    match message {
        Message::Increment => counter.value += 1,
    }
}

fn view(counter: &Counter) -> Element<Message> {
    button(text(counter.value)).on_press(Message::Increment).into()
}

fn main() -> iced::Result{
    iced::run("A cool counter", update, view)
}
