use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, CurrentScreen};

pub fn ui(frame: &mut Frame, app: &App) {
    match app.current_screen {
        CurrentScreen::Welcome => {
            render_welcome(frame, app);
        }
        CurrentScreen::LessonSelect => {
            render_lesson_select(frame, app);
        }
        CurrentScreen::Review => {
            render_review(frame, app);
        }
    }
}

fn render_welcome(frame: &mut Frame, app: &App) {
    let title = Line::from(" Genki Japanese Keyboard ".blue().bold());
    let instructions = Line::from(vec![" Welcome ".green().bold()]);
    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    let start = Paragraph::new(Text::styled(
        r"      |\      _,,,---,,_
    ZZZzz /,`.-'`'    -.  ;-;;,_
          |,4-  ) )-,_. ,\ (  `'-'
    '---''(_/--'  `-'\_)",
        Style::default().fg(Color::Green),
    ))
    .centered()
    .block(block);

    frame.render_widget(start, frame.area());
}

fn render_lesson_select(frame: &mut Frame, app: &App) {}

fn render_review(frame: &mut Frame, app: &App) {}
