use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::Book;
use crate::app::{App, CurrentScreen, CurrentSelection, TranslationDirection};

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
        .highlight_symbol("‣");
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
            section_state.select(app.context.section);
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
        .highlight_symbol("‣");

    frame.render_stateful_widget(section_list, selection_chunks[1], &mut section_state);
}

fn render_review(frame: &mut Frame, app: &App) {
    let [review_chunk, japanese_chunk, romanji_chunk] = Layout::vertical([
        Constraint::Percentage(35),
        Constraint::Percentage(35),
        Constraint::Percentage(30),
    ])
    .areas(frame.area());

    let [question_chunk, answer_selector_chunk] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(review_chunk);

    let lessons = app.book.get_lessons();
    assert!(app.context.lesson < lessons.len());
    let lesson = &lessons[app.context.lesson];
    assert!(app.context.section.unwrap() < lesson.sections.len());
    let section = &lesson.sections[app.context.section.unwrap()];

    let mut question_title = String::new();
    let lesson = Book::get_lesson(app.book.get_lessons(), app.context.lesson).unwrap();
    let section = Book::get_section(lesson, app.context.section.unwrap()).unwrap();
    let phrase = &section.phrases[app.context.phrase];
    match app.context.translation_direction {
        TranslationDirection::ToEN => {
            if let Some(kanji) = &phrase.kanji {
                question_title = format!(" Translate from Japanese\n'{}' - '{}'", phrase.jp, kanji);
            } else {
                question_title = format!(" Translate from Japanese\n'{}'", phrase.jp);
            }
        }
        TranslationDirection::ToJP => {
            question_title = format!(" Translate from English\n'{}'", phrase.en);
        }
    }
    let question_text = Paragraph::new(question_title)
        .block(Block::bordered().title(format!(" Lesson {} - {} ", lesson.index, section.name)))
        .wrap(Wrap { trim: true });
    frame.render_widget(question_text, question_chunk);

    let mut answer_title = String::new();
    if let Some(prev_phrase) = app.context.prev_phrase {
        assert!(app.context.prev_translation_direction != None);
        assert!(app.context.prev_answer != None);
        let prev_translation_direction = app.context.prev_translation_direction.unwrap();
        let prev_answer = app.context.prev_answer.as_ref().unwrap();
        let lesson = Book::get_lesson(app.book.get_lessons(), app.context.lesson).unwrap();
        let section = Book::get_section(lesson, app.context.prev_section.unwrap()).unwrap();
        let phrase = &section.phrases[prev_phrase];
        match prev_translation_direction {
            TranslationDirection::ToEN => {
                answer_title = format!(
                    "correct answer: '{}'\nyour answer:    '{}'",
                    phrase.en,
                    prev_answer.clone()
                );
            }
            TranslationDirection::ToJP => {
                if let Some(kanji) = &phrase.kanji {
                    answer_title = format!(
                        "correct answer: '{}' - '{}'\nyour answer:    '{}'",
                        phrase.jp,
                        kanji,
                        prev_answer.clone()
                    );
                } else {
                    answer_title = format!(
                        "correct answer: '{}'\nyour answer:    '{}'",
                        phrase.jp,
                        prev_answer.clone()
                    );
                }
            }
        }
    }
    let answer_text = Paragraph::new(answer_title)
        .block(Block::bordered().title(format!(" answer ")))
        .wrap(Wrap { trim: true });
    frame.render_widget(answer_text, answer_selector_chunk);

    let [kana_chunk, kanji_selector_chunk, kanji_chunk] = Layout::horizontal([
        Constraint::Percentage(45),
        Constraint::Percentage(10),
        Constraint::Percentage(45),
    ])
    .areas(japanese_chunk);

    // kana box with highlighting
    let kana = app.get_kana();
    let mut left = String::new();
    let mut middle = String::new();
    let mut right = String::new();
    if kana.chars().count() > 0 {
        assert!(app.kana_offset + app.kana_len <= kana.chars().count());
        assert!(app.kana_len >= 1);
        middle = kana
            .chars()
            .take(app.kana_offset + app.kana_len)
            .skip(app.kana_offset)
            .collect();
        if app.kana_offset > 0 {
            left = kana.chars().take(app.kana_offset).collect();
        }
        if app.kana_offset + app.kana_len < kana.chars().count() {
            right = kana
                .chars()
                .take(kana.chars().count())
                .skip(app.kana_offset + app.kana_len)
                .collect();
        }
    }

    // highlight selected kana
    let kana_formatted = Text::from(vec![Line::from(vec![
        Span::raw(left),
        Span::styled(&middle, Style::default().add_modifier(Modifier::REVERSED)),
        Span::raw(right),
    ])]);

    let kana_text = Paragraph::new(kana_formatted)
        .block(Block::bordered().title(" kana "))
        .wrap(Wrap { trim: true });
    frame.render_widget(kana_text, kana_chunk);

    // kanji selection
    let kanji = app.convert_to_kanji(&middle);
    let mut kanji_items = Vec::<ListItem>::new();
    for kanji_char in &kanji {
        kanji_items.push(ListItem::new(
            Line::from(Span::styled(
                format!(" {} ", kanji_char),
                Style::default().fg(Color::Yellow),
            ))
            .centered(),
        ));
    }

    let mut kanji_state = ListState::default();
    if kanji.len() > 0 {
        kanji_state.select(Some(0)); // TODO: kanji list navigation
    }

    let kanji_list = List::new(kanji_items)
        .block(Block::bordered().title(" kanji "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(kanji_list, kanji_selector_chunk, &mut kanji_state);

    // kanji text box
    let complete_text = Paragraph::new(app.get_kanji())
        .block(Block::bordered().title(" complete "))
        .wrap(Wrap { trim: true });
    frame.render_widget(complete_text, kanji_chunk);

    // romanji text box
    let romanji_text = Paragraph::new(app.get_romanji())
        .block(Block::bordered().title(" romanji "))
        .wrap(Wrap { trim: true });
    frame.render_widget(romanji_text, romanji_chunk);
}
