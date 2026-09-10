use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{App, CurrentScreen, CurrentSelection, TranslationDirection};

pub fn ui(frame: &mut Frame, app: &App) {
    match app.context.current_screen {
        CurrentScreen::Welcome => {
            render_welcome(frame, app);
        }
        CurrentScreen::Chat => {
            render_chat(frame, app);
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
    let subtitle = Line::from(vec![" げんき ".yellow().bold()]);
    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(subtitle.centered())
        .border_set(border::THICK);
    frame.render_widget(block, frame.area());

    let [top, bottom] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(frame.area());

    let ascii_cat = Paragraph::new(Text::styled(
        r"




      |\      _,,,---,,_
    ZZZzz /,`.-'`'    -.  ;-;;,_
          |,4-  ) )-,_. ,\ (  `'-'
    '---''(_/--'  `-'\_)",
        Style::default().fg(Color::Yellow),
    ))
    .centered();
    frame.render_widget(ascii_cat, top);

    let welcome_items = vec![
        ListItem::new(
            Line::from(Span::styled("Lessons", Style::default().fg(Color::Yellow))).centered(),
        ),
        ListItem::new(
            Line::from(Span::styled("Chat", Style::default().fg(Color::Yellow))).centered(),
        ),
    ];

    let mut welcome_state = ListState::default();
    welcome_state.select(Some(app.context.chat));

    let lesson_list =
        List::new(welcome_items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(lesson_list, bottom, &mut welcome_state);
}

fn render_keyboard(frame: &mut Frame, keyboard_rect: Rect, app: &App) {
    // kana box with highlighting
    let kana = app.get_kana();
    let mut left = String::new();
    let mut middle = String::new();
    let mut right = String::new();
    if kana.chars().count() > 0 {
        let cursor = app.get_cursor_pos();
        assert!(cursor.pos + cursor.len <= kana.chars().count());
        assert!(cursor.len >= 1);
        middle = kana
            .chars()
            .take(cursor.pos + cursor.len)
            .skip(cursor.pos)
            .collect();
        if cursor.pos > 0 {
            left = kana.chars().take(cursor.pos).collect();
        }
        if cursor.pos + cursor.len < kana.chars().count() {
            right = kana
                .chars()
                .take(kana.chars().count())
                .skip(cursor.pos + cursor.len)
                .collect();
        }
    }

    // highlight selected kana
    let kana_formatted = Text::from(vec![Line::from(vec![
        Span::raw(left),
        Span::styled(&middle, Style::default().add_modifier(Modifier::REVERSED)),
        Span::raw(right),
    ])]);

    let text_colour: Color;
    let title: String;
    if app.get_use_english() {
        text_colour = Color::White;
        title = " en ".to_string();
    } else {
        text_colour = Color::Yellow;
        title = " 日本語 ".to_string();
    };
    let text = Paragraph::new(kana_formatted)
        .block(
            Block::bordered()
                .title(title)
                .border_style(Style::default().fg(text_colour)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(text, keyboard_rect);

    assert!(
        (app.get_merge_kana().is_some() ^ app.get_kana_to_kanji().is_some())
            || (app.get_merge_kana().is_none() && app.get_kana_to_kanji().is_none())
    );
    // popup for merge kana option or kana to kanji options
    if let Some(merge_kana) = app.get_merge_kana() {
        let area = frame.area();
        let popup_layout = Layout::horizontal([
            Constraint::Length(frame.area().width - 6),
            Constraint::Length(6),
        ])
        .split(area);
        let popup_area = Layout::vertical([
            Constraint::Length((frame.area().height as f32 * 0.65) as u16),
            Constraint::Length(3),
        ])
        .split(popup_layout[1])[1];
        frame.render_widget(Clear, popup_area);

        let paragraph = Paragraph::new(
            Line::from(Span::styled(
                merge_kana,
                Style::default().fg(Color::Yellow).reversed(),
            ))
            .centered(),
        )
        .block(Block::bordered().border_set(border::DOUBLE).red())
        .bold();
        frame.render_widget(paragraph, popup_area);
    } else if let Some(kana_to_kanji) = app.get_kana_to_kanji() {
        let area = frame.area();
        let popup_layout = Layout::horizontal([
            Constraint::Length(frame.area().width - 6),
            Constraint::Length(6),
        ])
        .split(area);
        let popup_area = Layout::vertical([
            Constraint::Length(
                (frame.area().height as f32 * 0.65) as u16
                    - (kana_to_kanji.kanji_list.len() - 1) as u16,
            ),
            Constraint::Length(kana_to_kanji.kanji_list.len() as u16 + 2),
        ])
        .split(popup_layout[1])[1];
        frame.render_widget(Clear, popup_area);

        // use this method to only highlight backrgound directly behind text
        let items: Vec<ListItem> = kana_to_kanji
            .kanji_list
            .iter()
            .enumerate()
            .map(|(idx, kanji_char)| {
                if Some(kana_to_kanji.offset) == Some(idx) {
                    // Apply the highlight background directly to the text/span
                    ListItem::new(
                        Line::from(Span::styled(
                            kanji_char.to_string(),
                            Style::default()
                                .fg(Color::Yellow)
                                .bold()
                                .add_modifier(Modifier::REVERSED),
                        ))
                        .centered(),
                    )
                } else {
                    ListItem::new(
                        Line::from(Span::styled(
                            kanji_char.to_string(),
                            Style::default().fg(Color::Yellow).bold(),
                        ))
                        .centered(),
                    )
                }
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(kana_to_kanji.offset));

        let kanji_list = List::new(items).block(Block::bordered().border_set(border::DOUBLE).red());
        frame.render_stateful_widget(kanji_list, popup_area, &mut state);
    }
}

fn render_chat(frame: &mut Frame, app: &App) {
    let [message_chunk, keyboard] =
        Layout::vertical([Constraint::Percentage(70), Constraint::Percentage(30)])
            .areas(frame.area());

    // add debugging info here so we ca see it on the screen
    let messages = Paragraph::new(app.debug.clone())
        .block(Block::bordered().yellow())
        .wrap(Wrap { trim: true });
    frame.render_widget(messages, message_chunk);

    render_keyboard(frame, keyboard, app);
}

fn render_lesson_select(frame: &mut Frame, app: &App) {
    let [lesson_chunk, section_chunk] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(frame.area());

    // draw lesson selection
    let mut lesson_items = Vec::<ListItem>::new();
    for lesson in &app.book.lessons {
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
    if let CurrentSelection::Lesson = app.context.current_selection {
        lesson_border_thinkness = border::THICK;
    }

    let lesson_list = List::new(lesson_items)
        .block(
            Block::bordered()
                .title(Line::from(" Lessons ".yellow().bold()))
                .border_set(lesson_border_thinkness),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("‣");
    frame.render_stateful_widget(lesson_list, lesson_chunk, &mut lesson_state);

    // draw section selection
    let mut section_items = Vec::<ListItem>::new();
    let mut section_state = ListState::default();
    let mut section_border_thinkness = border::PLAIN;
    if let CurrentSelection::Section = app.context.current_selection {
        section_border_thinkness = border::THICK;
        assert!(app.context.lesson_idx < app.book.lessons.len());
        let lesson = &app.book.lessons[app.context.lesson_idx];
        for section in &lesson.sections {
            section_items.push(ListItem::new(Line::from(Span::styled(
                format!(" [{}] {} ", section_items.len(), section.name),
                Style::default().fg(Color::Yellow),
            ))));
        }
        section_state.select(app.context.section_idx);
    }

    let section_list = List::new(section_items)
        .block(
            Block::bordered()
                .title(Line::from(" Sections ".yellow().bold()))
                .border_set(section_border_thinkness),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("‣");

    frame.render_stateful_widget(section_list, section_chunk, &mut section_state);
}

fn render_review(frame: &mut Frame, app: &App) {
    let [review_chunk, keyboard_chunk] =
        Layout::vertical([Constraint::Percentage(70), Constraint::Percentage(30)])
            .areas(frame.area());

    let [question_chunk, answer_selector_chunk] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(review_chunk);

    let lessons = &app.book.lessons;
    assert!(app.context.lesson_idx < lessons.len());
    let lesson = &lessons[app.context.lesson_idx];
    assert!(app.context.section_idx.expect("missing section index") < lesson.sections.len());
    let section = &lesson.sections[app.context.section_idx.unwrap()];
    assert!(app.context.phrase_idx < section.phrases.len());
    let phrase = &section.phrases[app.context.phrase_idx];

    let question_title: String;
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
        let prev_translation_direction = app
            .context
            .prev_translation_direction
            .expect("previous translation direction not set");
        let prev_answer = app
            .context
            .prev_answer
            .as_ref()
            .expect("previous answer not set");
        assert!(app.context.lesson_idx < app.book.lessons.len());
        let lesson = &app.book.lessons[app.context.lesson_idx];
        assert!(
            app.context
                .prev_section_idx
                .expect("previous section index not set")
                < lesson.sections.len()
        );
        let section = &lesson.sections[app.context.prev_section_idx.unwrap()];
        assert!(prev_phrase_idx < section.phrases.len());
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
        .block(Block::bordered().title(" answer ").yellow())
        .wrap(Wrap { trim: true });
    frame.render_widget(answer_text, answer_selector_chunk);

    render_keyboard(frame, keyboard_chunk, app);
}
