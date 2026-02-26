mod app;
mod kana_converter;
mod kanji_converter;
mod lessons;
mod ui;

use rand::Rng;
use std::{error::Error, io};

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

use app::{App, CurrentScreen, CurrentSelection, TranslationDirection};
use ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(res) = res {
        if res {
            println!("done");
        }
    } else if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool>
where
    std::io::Error: From<<B as Backend>::Error>,
{
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                break;
            }
            match app.context.current_screen {
                CurrentScreen::Welcome => match key.code {
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {
                        app.context.current_screen = CurrentScreen::LessonSelect;
                    }
                },
                CurrentScreen::LessonSelect => match app.context.current_selection {
                    CurrentSelection::Lesson => match key.code {
                        KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Enter => {
                            app.context.current_screen = CurrentScreen::Review;
                            let translation_direction = rand::thread_rng().gen_range(0..=1);
                            if translation_direction == 0 {
                                app.context.translation_direction = TranslationDirection::ToJP;
                            } else {
                                app.context.translation_direction = TranslationDirection::ToEN;
                            }
                            assert!(app.context.lesson_idx < app.book.lessons.len());
                            let lesson = &app.book.lessons[app.context.lesson_idx];
                            app.context.section_idx =
                                Some(rand::thread_rng().gen_range(0..lesson.sections.len()));
                            let section = &lesson.sections[app.context.section_idx.unwrap()];
                            app.context.phrase_idx =
                                rand::thread_rng().gen_range(0..section.phrases.len());
                            app.context.randomize_section = true;
                        }
                        KeyCode::Down => {
                            if app.context.lesson_idx + 1 >= app.book.lessons.len() {
                                app.context.lesson_idx = 0;
                            } else {
                                app.context.lesson_idx += 1;
                            }
                        }
                        KeyCode::Up => {
                            if app.context.lesson_idx == 0 {
                                app.context.lesson_idx = app.book.lessons.len() - 1;
                            } else {
                                app.context.lesson_idx -= 1;
                            }
                        }
                        KeyCode::Right => {
                            app.context.current_selection = CurrentSelection::Section;
                            app.context.section_idx = Some(0);
                            app.context.randomize_section = false;
                        }
                        _ => {}
                    },
                    CurrentSelection::Section => match key.code {
                        KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Enter => {
                            app.context.current_screen = CurrentScreen::Review;
                            let translation_direction = rand::thread_rng().gen_range(0..=1);
                            if translation_direction == 0 {
                                app.context.translation_direction = TranslationDirection::ToJP;
                            } else {
                                app.context.translation_direction = TranslationDirection::ToEN;
                            }
                            assert!(app.context.lesson_idx < app.book.lessons.len());
                            let lesson = &app.book.lessons[app.context.lesson_idx];
                            assert!(
                                app.context.section_idx.expect("section index not set")
                                    < lesson.sections.len()
                            );
                            let section = &lesson.sections[app.context.section_idx.unwrap()];
                            app.context.phrase_idx =
                                rand::thread_rng().gen_range(0..section.phrases.len());
                        }
                        KeyCode::Down => {
                            assert!(app.context.lesson_idx < app.book.lessons.len());
                            let lesson = &app.book.lessons[app.context.lesson_idx];
                            if app.context.section_idx.expect("section index not set") + 1
                                >= lesson.sections.len()
                            {
                                app.context.section_idx = Some(0);
                            } else {
                                app.context.section_idx =
                                    Some(app.context.section_idx.unwrap() + 1);
                            }
                        }
                        KeyCode::Up => {
                            assert!(app.context.lesson_idx < app.book.lessons.len());
                            let lesson = &app.book.lessons[app.context.lesson_idx];
                            if app.context.section_idx.expect("section index not set") == 0 {
                                app.context.section_idx = Some(lesson.sections.len() - 1);
                            } else {
                                app.context.section_idx =
                                    Some(app.context.section_idx.unwrap() - 1);
                            }
                        }
                        KeyCode::Left => {
                            app.context.current_selection = CurrentSelection::Lesson;
                            app.context.section_idx = None;
                        }
                        _ => {}
                    },
                },
                CurrentScreen::Review => match key.code {
                    KeyCode::Esc => {
                        app.context.current_screen = CurrentScreen::LessonSelect;
                        app.context.current_selection = CurrentSelection::Lesson;
                        app.context.lesson_idx = 0;
                        app.context.section_idx = None;
                        app.context.prev_section_idx = None;
                        app.context.prev_phrase_idx = None;
                        app.context.prev_translation_direction = None;
                        app.context.prev_answer = None;
                        app.romanji.clear();
                        app.kana.clear();
                        app.kanji.clear();
                        app.highlighted_kanji.clear();
                        app.kana_offset = 0;
                        app.kana_len = 1;
                    }
                    KeyCode::Enter => {
                        app.context.prev_section_idx = app.context.section_idx;
                        app.context.prev_phrase_idx = Some(app.context.phrase_idx);
                        app.context.prev_translation_direction =
                            Some(app.context.translation_direction);
                        if let Some(prev_translation_direction) =
                            app.context.prev_translation_direction
                        {
                            match prev_translation_direction {
                                TranslationDirection::ToEN => {
                                    app.context.prev_answer = Some(app.romanji.clone());
                                }
                                TranslationDirection::ToJP => {
                                    app.context.prev_answer = Some(app.kanji.clone());
                                }
                            }
                        }
                        let translation_direction = rand::thread_rng().gen_range(0..=1);
                        if translation_direction == 0 {
                            app.context.translation_direction = TranslationDirection::ToJP;
                        } else {
                            app.context.translation_direction = TranslationDirection::ToEN;
                        }
                        assert!(app.context.lesson_idx < app.book.lessons.len());
                        let lesson = &app.book.lessons[app.context.lesson_idx];
                        if app.context.randomize_section == true {
                            app.context.section_idx =
                                Some(rand::thread_rng().gen_range(0..lesson.sections.len()));
                        }
                        assert!(
                            app.context.section_idx.expect("section index not set")
                                < lesson.sections.len()
                        );
                        let section = &lesson.sections[app.context.section_idx.unwrap()];
                        app.context.phrase_idx =
                            rand::thread_rng().gen_range(0..section.phrases.len());
                        app.romanji.clear();
                        app.kana.clear();
                        app.kanji.clear();
                        app.highlighted_kanji.clear();
                        app.kana_offset = 0;
                        app.kana_len = 1;
                    }
                    KeyCode::Tab => {
                        if app.get_kana().chars().count() > 0 {
                            assert!(
                                app.kana_offset < app.get_kana().chars().count()
                                    && app.kana_offset + app.kana_len
                                        <= app.get_kana().chars().count()
                            );
                            app.push_kanji_offset((app.kana_offset, app.kana_len, 0)); // TODO: kanji selection
                        }
                    }
                    KeyCode::Right => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            if app.kana_offset + app.kana_len < app.get_kana().chars().count() {
                                app.kana_len += 1;
                            }
                        } else {
                            if app.kana_offset + app.kana_len < app.get_kana().chars().count() {
                                app.kana_offset += 1;
                            }
                            app.kana_len = 1;
                        }
                    }
                    KeyCode::Left => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            if app.kana_len > 1 {
                                app.kana_len -= 1;
                            }
                        } else {
                            if app.kana_offset > 0 {
                                app.kana_offset -= 1;
                            }
                            app.kana_len = 1;
                        }
                    }
                    KeyCode::Char(value) => {
                        app.push_char(value);
                        // when last charachter dissapears do to kana conversion
                        if app.kana_offset + app.kana_len > app.get_kana().chars().count() {
                            app.kana_offset -=
                                app.kana_offset + app.kana_len - app.get_kana().chars().count();
                        }
                    }
                    KeyCode::Backspace => {
                        app.pop_char();
                        // undo highlighted all chars if removing highlighted char
                        if app.get_kana().chars().count() > 0
                            && app.kana_len > 1
                            && app.kana_offset + app.kana_len > app.get_kana().chars().count()
                        {
                            app.kana_len -= 1;
                        }
                        // move cursor back if cursor is at end of string
                        if app.get_kana().chars().count() > 0
                            && app.kana_offset + app.kana_len > app.get_kana().chars().count()
                        {
                            app.kana_offset -=
                                app.kana_offset + app.kana_len - app.get_kana().chars().count();
                        }
                    }
                    _ => {}
                },
            }
        }
        app.update_kanji();
    }
    return Ok(true);
}
