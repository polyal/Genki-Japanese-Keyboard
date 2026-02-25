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
    let title = Line::from(" Genki Japanese Keyboard ".yellow().bold());
    let instructions = Line::from(vec![" げんき ".yellow().bold()]);
    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    let start = Paragraph::new(Text::styled(
        r"




      |\      _,,,---,,_
    ZZZzz /,`.-'`'    -.  ;-;;,_
          |,4-  ) )-,_. ,\ (  `'-'
    '---''(_/--'  `-'\_)",
        Style::default().fg(Color::Yellow),
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
    lesson_state.select(Some(app.context.lesson_idx));

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
                .title(Line::from(" Lessons ".yellow().bold()))
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
            let lesson =
                Book::get_lesson(app.book.get_lessons(), app.context.lesson_idx).expect(&format!(
                    "lesson index [{}.{}) out of range",
                    app.context.lesson_idx,
                    app.book.get_lessons().len()
                ));
            for section in &lesson.sections {
                section_items.push(ListItem::new(Line::from(Span::styled(
                    format!(" [{}] {} ", section_items.len(), section.name),
                    Style::default().fg(Color::Yellow),
                ))));
            }
            section_state.select(app.context.section_idx);
        }
        _ => {}
    }

    let section_list = List::new(section_items)
        .block(
            Block::bordered()
                .title(Line::from(" Sections ".yellow().bold()))
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
    assert!(app.context.lesson_idx < lessons.len());
    let lesson = &lessons[app.context.lesson_idx];
    assert!(app.context.section_idx.expect("section index missing") < lesson.sections.len());
    let section = &lesson.sections[app.context.section_idx.unwrap()];

    let mut question_title = String::new();
    let lesson = Book::get_lesson(app.book.get_lessons(), app.context.lesson_idx).expect(&format!(
        "lesson index [{}.{}) out of range",
        app.context.lesson_idx,
        app.book.get_lessons().len()
    ));
    let section = Book::get_section(lesson, app.context.section_idx.unwrap()).expect(&format!(
        "section index [{}.{}) out of range",
        app.context.section_idx.unwrap(),
        lesson.sections.len()
    ));
    let phrase = &section.phrases[app.context.phrase_idx];
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
        .light_yellow()
        .block(
            Block::bordered()
                .title(format!(" Lesson {} - {} ", lesson.index, section.name))
                .yellow(),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(question_text, question_chunk);

    let mut answer_title = String::new();
    if let Some(prev_phrase_idx) = app.context.prev_phrase_idx {
        assert!(app.context.prev_translation_direction != None);
        assert!(app.context.prev_answer != None);
        let prev_translation_direction = app
            .context
            .prev_translation_direction
            .expect("previous translation direction not set");
        let prev_answer = app
            .context
            .prev_answer
            .as_ref()
            .expect("previous answer not set");
        let lesson =
            Book::get_lesson(app.book.get_lessons(), app.context.lesson_idx).expect(&format!(
                "lesson index [{}.{}) out of range",
                app.context.lesson_idx,
                app.book.get_lessons().len()
            ));
        let section = Book::get_section(
            lesson,
            app.context
                .prev_section_idx
                .expect("previous section index not set"),
        )
        .expect(&format!(
            "previous section index [{}.{}) out of range",
            app.context.prev_section_idx.unwrap(),
            lesson.sections.len()
        ));
        let phrase = &section.phrases[prev_phrase_idx];
        match prev_translation_direction {
            TranslationDirection::ToEN => {
                if let Some(kanji) = &phrase.kanji {
                    answer_title = format!(
                        " Translate from Japanese\n'{}' - '{}'\n\ncorrect answer: '{}'\nyour answer:    '{}'",
                        phrase.jp,
                        kanji,
                        phrase.en,
                        prev_answer.clone()
                    );
                } else {
                    answer_title = format!(
                        " Translate from Japanese\n'{}'\n\ncorrect answer: '{}'\nyour answer:    '{}'",
                        phrase.jp,
                        phrase.en,
                        prev_answer.clone()
                    );
                }
            }
            TranslationDirection::ToJP => {
                if let Some(kanji) = &phrase.kanji {
                    answer_title = format!(
                        " Translate from English\n'{}'\n\ncorrect answer: '{}' - '{}'\nyour answer:    '{}'",
                        phrase.en,
                        phrase.jp,
                        kanji,
                        prev_answer.clone()
                    );
                } else {
                    answer_title = format!(
                        " Translate from English\n'{}'\n\ncorrect answer: '{}'\nyour answer:    '{}'",
                        phrase.en,
                        phrase.jp,
                        prev_answer.clone()
                    );
                }
            }
        }
    }
    let answer_text = Paragraph::new(answer_title)
        .light_yellow()
        .block(Block::bordered().title(format!(" answer ")).yellow())
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
        .light_yellow()
        .block(Block::bordered().title(" kana ").yellow())
        .wrap(Wrap { trim: true });
    frame.render_widget(kana_text, kana_chunk);

    // kanji selection
    let kanji = &app.highlighted_kanji;
    let mut kanji_items = Vec::<ListItem>::new();
    for kanji_char in kanji {
        kanji_items.push(ListItem::new(
            Line::from(Span::styled(
                format!(" {} ", kanji_char),
                Style::default().fg(Color::LightYellow),
            ))
            .centered(),
        ));
    }

    let mut kanji_state = ListState::default();
    if kanji.len() > 0 {
        kanji_state.select(Some(0)); // TODO: kanji list navigation
    }

    let kanji_list = List::new(kanji_items)
        .dark_gray()
        .block(Block::bordered().title(" kanji ").yellow())
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(kanji_list, kanji_selector_chunk, &mut kanji_state);

    // kanji text box
    let complete_text = Paragraph::new(app.get_kanji())
        .light_yellow()
        .block(Block::bordered().title(" complete ").yellow())
        .wrap(Wrap { trim: true });
    frame.render_widget(complete_text, kanji_chunk);

    // romanji text box
    let romanji_text = Paragraph::new(app.get_romanji())
        .block(Block::bordered().title(" romanji "))
        .wrap(Wrap { trim: true });
    frame.render_widget(romanji_text, romanji_chunk);
}
