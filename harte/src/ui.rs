use std::{
    io::{self, Stdout},
    time::Duration,
};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, TryRecvError};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::event::KeyEventKind;
use ratatui::{terminal::*, backend::*, style::*, layout::*, layout::Constraint::*, widgets::*};
use ratatui::text::{Line, Span};
use crate::{FileStatus, TestStatus};
use crate::list_management::StatefulList;

/// This is a bare minimum example. There are many approaches to running an application loop, so
/// this is not meant to be prescriptive. It is only meant to demonstrate the basic setup and
/// teardown of a terminal application.
///
/// A more robust application would probably want to handle errors and ensure that the terminal is
/// restored to a sane state before exiting. This example does not do that. It also does not handle
/// events or update the application state. It just draws a greeting and exits when the user
/// presses 'q'.
pub fn main(receiver: Receiver<FileStatus>) -> Result<()> {
    let mut terminal = setup_terminal().context("setup failed")?;
    run(&mut terminal, receiver).context("app loop failed")?;
    restore_terminal(&mut terminal).context("restore terminal failed")?;
    Ok(())
}

/// Setup the terminal. This is where you would enable raw mode, enter the alternate screen, and
/// hide the cursor. This example does not handle errors. A more robust application would probably
/// want to handle errors and ensure that the terminal is restored to a sane state before exiting.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
}

/// Restore the terminal. This is where you disable raw mode, leave the alternate screen, and show
/// the cursor.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}

struct AppStatus {
    statuses: Vec<FileStatus>,
    passed: u64,
    failed: u64,
    skipped: u64,
    // map "01 aa bc" to a list of errors
    failed_details: HashMap<String, Vec<String>>,

    state_column0: StatefulList,
    state_column1: StatefulList,
    state_column2: StatefulList,

    focused_column: usize,   // 0 or 1
}

impl AppStatus {
    fn selected_opcode(&self) -> Option<&str> {
        if let Some(selection) = self.state_column0.state.selected() {
            Some(&self.state_column0.items[selection])
        } else {
            None
        }
    }
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            statuses: {
                let mut result: Vec<FileStatus> = Vec::new();
                for i in 0..=255 {
                    result.push(FileStatus::NotStarted(i as u8));
                }
                result
            },
            passed: 0,
            failed: 0,
            skipped: 0,
            failed_details: HashMap::new(),
            state_column0: StatefulList::default(),
            state_column1: StatefulList::default(),
            state_column2: StatefulList::default(),
            focused_column: 0,
        }
    }
}

struct Containers {
    data: Vec<(u16, u16, u16, u16)>,
}

impl Containers {
    fn new() -> Self {
        let width_first_panel = 56;
        let height = 36;
        let width_second_panel = 60;
        let height_second_panel = 5;
        let height_third_panel = 20;
        let height_fourth_panel = height_second_panel + height_third_panel;

        Self {
            data: vec![
                (0, 0, width_first_panel, height),
                (width_first_panel, 0, width_second_panel, height_second_panel),
                (width_first_panel, height_second_panel, width_second_panel, height_third_panel),
                (width_first_panel, height_fourth_panel, width_second_panel, height - height_fourth_panel)
            ]
        }
    }

    fn all(&self) -> Vec<Rect> {
        self.data.iter().map(|t| Rect::new(t.0, t.1, t.2, t.3)).collect::<Vec<Rect>>()
    }
}

/// Run the application loop. This is where you would handle events and update the application
/// state. This example exits when the user presses 'q'. Other styles of application loops are
/// possible, for example, you could have multiple application states and switch between them based
/// on events, or you could have a single application state and update it based on events.
fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, receiver: Receiver<FileStatus>)
        -> Result<()> {
    let mut app_status = AppStatus::default();
    let containers = Containers::new().all();

    loop {
        let mut iter = containers.iter();

        terminal.draw(|rect| {
            //
            // First panel
            //
            let first_panel = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                     Length(100),
                     Min(0),
                 ]
                 .as_ref(),
            )
            .split(*iter.next().unwrap());
            let hex_style = Style::default().fg(Color::Yellow);

            //
            // First panel, first widget
            //
            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(Span::styled(
                "   0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F", hex_style)));
            let mut spans: [Vec<Span>;2] = [Vec::new(), Vec::new()];
            spans[0].push(Span::styled(" 0 ", hex_style));
            spans[1].push(Span::raw("   "));
            for i in 0..=255 {
                if i % 16 == 0 && i > 0 {
                    lines.push(Line::from(spans[0].clone()));
                    lines.push(Line::from(spans[1].clone()));
                    spans = [Vec::new(), Vec::new()];
                    spans[0].push(Span::styled(format!(" {:1X} ", i / 16), hex_style));
                    spans[1].push(Span::raw("   "));
                }
                match app_status.statuses.get(i).unwrap() {
                    FileStatus::Exit() => {}
                    FileStatus::NotStarted(_) => {
                        spans[0].push(Span::raw("   "));
                        spans[1].push(Span::raw(" . "));
                    }
                    FileStatus::Completed(_, _, failed, skipped, _) => {
                        let color = Style::new().bg(
                            if *failed > 0 {
                                Color::Red
                            } else if *skipped == 10_000 {
                                Color::Rgb(0xff, 0xf3, 0xb0)
                            } else {
                                Color::Rgb(0x2a, 0x72, 0x21)
                            });
                        spans[0].push(Span::styled("   ", color));
                        spans[1].push(Span::styled("   ", color));
                    }
                }
            }
            lines.push(Line::from(spans[0].clone()));
            lines.push(Line::from(spans[1].clone()));
            let title = Paragraph::new(lines)
                // .style(Style::default().fg(Color::LightCyan))
                // .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("  Harte Tests  ")
                        .border_type(BorderType::Plain),
                );

            rect.render_widget(title, first_panel[0]);

            //
            // Second panel (Passed / Failed / Skipped)
            //
            let second_panel = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Length(5),
                    Min(0),
                ]
                .as_ref(),
            ).split(*iter.next().unwrap());

            //
            // Second panel, first widget
            //
            let lines = vec![
                Line::from(format!("Passed: {}", app_status.passed)),
                Line::from(format!("Failed: {}", app_status.failed)),
                Line::from(format!("Skipped: {}", app_status.skipped)),
            ];
            let percentage = 100.0 * app_status.passed as f64 /
                (app_status.passed as f64 + app_status.failed as f64 + app_status.skipped as f64);
            let status = Paragraph::new(lines)
                .style(Style::default().fg(Color::LightCyan))
                // .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title(format!("  Status  ({:.0}%) ", percentage))
                        .border_type(BorderType::Plain),
                );
            rect.render_widget(status, second_panel[0]);

            //
            // The three columns
            //

            fn create_list(items: &Vec<String>, index: usize, selected_index: usize,
                    title: Option<String>) -> List {
                let border_type = if index == selected_index {
                    BorderType::Thick
                } else {
                    BorderType::Plain
                };

                let items: Vec<ListItem> = items.iter()
                    .map(|i| ListItem::new(Span::raw(i))).collect();

                let mut block = Block::default().borders(Borders::ALL).border_type(border_type);
                if let Some(title) = title {
                    block = block.title(title);
                }
                List::new(items)
                    .block(block)
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().bg(Color::Red))
            }

            //
            // The layouts for the three columns
            //
            let columns_panel = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                     Length(10),
                     Min(0)
                 ]
                 .as_ref(),
                ).split(*iter.next().unwrap());

            let mut column_index = 0_usize;

            //
            // First column
            //
            let list_of_failed_opcodes = create_list(&app_status.state_column0.items, column_index,
                                                     app_status.focused_column, None);
            rect.render_stateful_widget(list_of_failed_opcodes, columns_panel[column_index],
                &mut app_status.state_column0.state);

            //
            // Second column
            //
            column_index += 1;
            let mut items: Vec<String> = Vec::new();

            // Populate the items
            // Only do that if an opcode is selected in the first column
            if let Some(opcode_index) = app_status.state_column0.state.selected() {
                let opcode = &app_status.state_column0.items[opcode_index];
                // log::info!("Selected opcode: {}", opcode);
                for failures in app_status.failed_details.iter() {
                    let name = failures.0;
                    if name.starts_with(opcode) {
                        items.push(name.to_string());
                    }
                }
                app_status.state_column1.items = items;
            }

            let list_of_failed_tests = create_list(&app_status.state_column1.items, column_index,
                                                   app_status.focused_column, None);
            rect.render_stateful_widget(list_of_failed_tests, columns_panel[column_index],
                                        &mut app_status.state_column1.state);

            //
            // Third column
            //

            let errors_panel = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                     Min(0)
                 ]
                 .as_ref(),
                ).split(*iter.next().unwrap());

            column_index += 1;
            let mut items: Vec<String> = Vec::new();
            // Populate the third column only if a test is selected in the second column
            if let Some(index) = app_status.state_column1.state.selected() {
                let selected_test = &app_status.state_column1.items[index];
                if let Some(errors) = app_status.failed_details.get(selected_test) {
                    for error in errors {
                        items.push(error.to_string());
                    }
                }
            }
            app_status.state_column2.items = items;

            let list_of_error_messages = create_list(&app_status.state_column2.items, column_index,
                                     app_status.focused_column, Some("  Test errors  ".to_string()));
            rect.render_widget(list_of_error_messages, errors_panel[0]);

        }).unwrap();

        match receiver.try_recv() {
            Ok(test_status) => {
                let opcode = match test_status {
                    FileStatus::Exit() => { 0 }
                    FileStatus::NotStarted(opcode) => { opcode },
                    FileStatus::Completed(opcode, passed, failed, skipped, ref statuses) => {
                        // log::info!("Received Completed: {} {:?}", failed, statuses);
                        app_status.passed += passed;
                        app_status.failed += failed;
                        app_status.skipped += skipped;
                        if failed > 0 {
                            app_status.state_column0.items.push(format!("{:02x}", opcode));
                            if app_status.state_column0.items.len() == 1 {
                                app_status.state_column0.next();
                                app_status.state_column1.next();
                                app_status.state_column2.next();
                            }
                        }
                        for status in statuses {
                            let new_vec: Vec<String> = Vec::new();
                            // log::info!("Maybe inserting {}", status.name());
                            let stored_errors: &mut Vec<String> =
                                    app_status.failed_details.entry(status.name())
                                        .or_insert(new_vec);

                            // log::info!("Looking at status {:?}", status);
                            match status {
                                TestStatus::Failed(_, _, errors) => {
                                    for e in errors {
                                        // log::info!("Test {}, inserted error {:?}", status.name(), e);
                                        stored_errors.push(e.clone());
                                    }
                                }
                                _ => {}
                            }

                        }
                        opcode
                    },
                } as usize;
                app_status.statuses[opcode] = test_status;
                // println!("Test status: {:?}", test_status);
            }
            Err(TryRecvError::Disconnected) => {}
            Err(TryRecvError::Empty) => {}
        }

        let mut should_quit = false;

        fn inc_wrap(value: usize, max: usize) -> usize {
            if value < max - 1 { value + 1 }
            else { 0 }
        }

        if event::poll(Duration::from_millis(1)).context("event poll failed")? {
            if let Event::Key(key) = event::read().context("event read failed")? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => { should_quit = true }
                        KeyCode::Tab => {
                            app_status.focused_column = inc_wrap(
                                app_status.focused_column, 2);
                        }
                        KeyCode::Down => {
                            if app_status.focused_column == 0 {
                                app_status.state_column0.next();
                                let sel = app_status.state_column0.state.selected();
                                log::info!("  Current opcode selection: {:?}", sel);
                            } else {
                                if let Some(op) = app_status.selected_opcode() {
                                    log::info!("Opcode selected: {}, moving to next test", op);
                                    app_status.state_column1.next();
                                    let sel = app_status.state_column1.state.selected();
                                    log::info!("  Current selection: {:?}", sel);
                                }
                            }
                        }
                        KeyCode::Up => {
                            if app_status.focused_column == 0 {
                                app_status.state_column0.previous();
                            } else {
                                if let Some(_) = app_status.selected_opcode() {
                                    app_status.state_column1.previous();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if should_quit {
            break;
        }
    }
    Ok(())
}
