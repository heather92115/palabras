use iced::widget::{Button, Column, Container, Text, TextInput};
use iced::{Alignment, Element, Length, Sandbox, Theme};
use iced::alignment::{Horizontal, Vertical};
use crate::gui::study_words::{ManageStudySet, StudySet};


pub enum CardMode {
    UserEntryView,
    MatchOutcomeView,
    CompletedSet
}
pub struct CardText {
    head_line: String,
    prompt: String,
}

pub struct Card {
    study_set: StudySet,
    mode: CardMode,
    text: CardText,
    user_response: String,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Message {
    Response(String),
    CheckMatch,
    NextCard,
}

static STUDY_CARD_NUM: i64 = 10;
impl Sandbox for Card {
    type Message = Message;

    fn new() -> Self {

        let mut study_set = StudySet::default();
        study_set.next_study_set(STUDY_CARD_NUM.clone());
        let prompt = study_set.determine_word_prompt();
        let head_line =  if study_set.has_vocab_ready() {
            format!("Your vocabulary words are ready! ({} to go)", study_set.remaining_study_pairs())
        } else {
            "You don't have any vocabulary words setup yet!".to_string()
        };

        Self {
            study_set,
            mode: CardMode::UserEntryView,
            text: CardText {
                head_line,
                prompt,
            },
            user_response: "".to_string(),
        }
    }

    fn title(&self) -> String {
        String::from("DuoLingo Vocabulary Cards")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Response(entered) => {
                self.user_response = entered;
            }
            Message::CheckMatch => {

                self.text.prompt = self.study_set.check_pair_distance(&self.user_response);
                self.mode = CardMode::MatchOutcomeView;
            }
            Message::NextCard => {
                self.study_set.next();
                self.text.head_line = format!("Your vocabulary words are ready! ({} to go)", self.study_set.remaining_study_pairs());
                self.text.prompt = self.study_set.determine_word_prompt();
                self.mode = if self.study_set.has_vocab_ready() { CardMode::UserEntryView } else { CardMode::CompletedSet }
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {

        let mut content = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(Text::new(&self.text.head_line).size(26))
            .spacing(80)
            .push(Text::new(&self.text.prompt).size(20));

        match self.mode {
            CardMode::UserEntryView => {
                content =
                    content.push(
                        TextInput::new(
                            "Enter your response here...",
                            &self.user_response
                        )
                        .size(30)
                        .on_input(Message::Response)
                        .on_submit(Message::CheckMatch)
                    )
                        .push(Button::new("Check").on_press(Message::CheckMatch));
            },
            CardMode::MatchOutcomeView => {
                content =
                    content
                        .push(Button::new("Next").on_press(Message::NextCard))
            },
            _ => ()
        }

        Container::new(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}