use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::Book;
use crate::app::{App, CurrentScreen, CurrentSelection};

pub fn ui(frame: &mut Frame, app: &App) {
    match app.context.current_screen {
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

fn render_lesson_select(frame: &mut Frame, app: &App) {
    let selection_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    // draw lesson selection
    let mut lesson_items = Vec::<ListItem>::new();
    for lesson in app.book.get_lessons() {
        lesson_items.push(ListItem::new(Line::from(Span::styled(
            format!(
                " [{}] {} - {} ",
                lesson.index, lesson.name_en, lesson.name_jp
            ),
            Style::default().fg(Color::Yellow),
        ))));
    }

    let mut lesson_state = ListState::default();
    lesson_state.select(Some(app.context.lesson));

    let mut lesson_border_thinkness = border::PLAIN;
    match app.context.current_selection {
        CurrentSelection::Lesson => {
            lesson_border_thinkness = border::THICK;
        }
        _ => {}
    }

    let lesson_list = List::new(lesson_items)
        .block(
            Block::bordered()
                .title(Line::from(" Lessons ".blue().bold()))
                .border_set(lesson_border_thinkness),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    frame.render_stateful_widget(lesson_list, selection_chunks[0], &mut lesson_state);

    // draw section selection
    let mut section_items = Vec::<ListItem>::new();
    let mut section_state = ListState::default();
    let mut section_border_thinkness = border::PLAIN;
    match app.context.current_selection {
        CurrentSelection::Section => {
            section_border_thinkness = border::THICK;
            let lesson = Book::get_lesson(app.book.get_lessons(), app.context.lesson).unwrap();
            for section in &lesson.sections {
                section_items.push(ListItem::new(Line::from(Span::styled(
                    format!(" [{}] {} ", section_items.len(), section.name),
                    Style::default().fg(Color::Yellow),
                ))));
            }
            section_state.select(Some(app.context.section));
        }
        _ => {}
    }

    let section_list = List::new(section_items)
        .block(
            Block::bordered()
                .title(Line::from(" Sections ".blue().bold()))
                .border_set(section_border_thinkness),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    frame.render_stateful_widget(section_list, selection_chunks[1], &mut section_state);
}

fn render_review(frame: &mut Frame, app: &App) {
    let [review_chunk, japanese_chunk, romanji_chunk] = Layout::vertical([
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(40),
    ])
    .areas(frame.area());

    let [question_chunk, answer_selectior_chunk] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(review_chunk);
    frame.render_widget(Block::bordered().title(" question "), question_chunk);
    frame.render_widget(Block::bordered().title(" answer "), answer_selectior_chunk);

    let [kana_chunk, kanji_selectior_chunk, kanji_chunk] = Layout::horizontal([
        Constraint::Percentage(45),
        Constraint::Percentage(10),
        Constraint::Percentage(45),
    ])
    .areas(japanese_chunk);

    let kana_text = Paragraph::new(app.get_kana())
        .block(Block::bordered().title(" kana "))
        .wrap(Wrap { trim: true });
    frame.render_widget(kana_text, kana_chunk);

    frame.render_widget(Block::bordered().title(" kanji "), kanji_selectior_chunk);
    frame.render_widget(Block::bordered().title(" complete "), kanji_chunk);

    let romanji_text = Paragraph::new(app.get_romanji())
        .block(Block::bordered().title(" romanji "))
        .wrap(Wrap { trim: true });
    frame.render_widget(romanji_text, romanji_chunk);
}
