use crate::chat::{Message, Peer};
use crate::keyboard::Keyboard;
use crate::lessons::Book;

use std::collections::{HashSet, VecDeque};

const MAX_MESSAGES: usize = 50;

pub enum CurrentScreen {
    Welcome,
    Chat,
    LessonSelect,
    Review,
}

pub enum CurrentSelection {
    Lesson,
    Section,
}

pub enum ChatSelection {
    Chat,
    Popup,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TranslationDirection {
    ToEN,
    ToJP,
}

pub struct Context {
    pub current_screen: CurrentScreen,
    pub current_selection: CurrentSelection,

    pub chat: usize,
    pub chat_screen: ChatSelection,
    pub host: usize,

    pub lesson_idx: usize,
    pub section_idx: Option<usize>,
    pub phrase_idx: usize,
    pub translation_direction: TranslationDirection,

    pub prev_section_idx: Option<usize>,
    pub prev_phrase_idx: Option<usize>,
    pub prev_translation_direction: Option<TranslationDirection>,
    pub prev_answer: Option<String>,

    pub randomize_section: bool,
    pub asked_questions: Vec<HashSet<usize>>,
}

impl Context {
    fn new() -> Self {
        Context {
            current_screen: CurrentScreen::Welcome,
            current_selection: CurrentSelection::Lesson,
            chat: 0,
            chat_screen: ChatSelection::Popup,
            host: 0,
            lesson_idx: 0,
            section_idx: None,
            phrase_idx: 0,
            translation_direction: TranslationDirection::ToEN,
            prev_section_idx: None,
            prev_phrase_idx: None,
            prev_translation_direction: None,
            prev_answer: None,
            randomize_section: false,
            asked_questions: Vec::<HashSet<usize>>::new(),
        }
    }
}

pub struct App {
    pub book: Book,
    pub context: Context,

    pub keyboard: Keyboard,

    peer: Peer,
    messages: VecDeque<Message>,
}

impl App {
    pub fn new() -> Self {
        App {
            book: Book::new(),
            context: Context::new(),
            keyboard: Keyboard::new(),
            peer: Peer::new(),
            messages: VecDeque::new(),
        }
    }

    pub fn create_server(&mut self) {
        let _ = self.peer.host();
    }

    pub fn create_client(&mut self) {
        let _ = self.peer.connect(self.keyboard.get_romaji().trim());
    }

    pub fn push_message(&mut self) {
        if !self.keyboard.get_kana().is_empty() {
            let message = Message::new(self.keyboard.get_kana());
            if self.is_online() {
                let _ = self.peer.send_message(&message);
            }
            self.messages.push_back(message);
        }
    }

    pub fn check_received_message(&mut self) -> bool {
        // prune first
        if self.messages.len() > MAX_MESSAGES {
            for _ in 0..self.messages.len() - MAX_MESSAGES {
                self.messages.pop_front();
            }
        }
        if self.peer.peek_message() {
            self.messages.push_back(self.peer.pop_message());
            return true;
        }
        return false;
    }

    pub fn get_messages(&self) -> &VecDeque<Message> {
        return &self.messages;
    }

    pub fn is_online(&self) -> bool {
        return self.peer.is_online();
    }

    pub fn reset_peer(&mut self) {
        self.peer.reset_connection();
        self.messages.clear();
    }

    pub fn get_peer(&self) -> String {
        return self.peer.get_peer();
    }
}
