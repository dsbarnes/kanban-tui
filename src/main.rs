mod util;

#[allow(unused_imports)]
use crate::util::{
    StatefulList, TabsState,
    event::{ Event, Events },
};

#[allow(unused_imports)]
use std::{error::Error, io};

#[allow(unused_imports)]
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};

#[allow(unused_imports)]
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Paragraph, Block, Borders, List, ListItem},
    Terminal,
};

enum InputMode {
    Normal,
    Input,
    Move,
}

#[derive(Debug, Clone)]
struct Ticket {
    title: String,
    body: String,
    points: u8,
}
impl Ticket {
    pub fn new(title: String) -> Ticket {
        Ticket {
            title,
            body: String::new(), 
            points: 0,
        }
    }
}

struct App {
    input: String,
    input_mode: InputMode,
    kanban: Vec<StatefulList<Ticket>>,
    current_board: u8,
}

impl App {
    pub fn new() -> App {
        App{
            input: String::new(),
            input_mode: InputMode::Normal,
            kanban: vec![StatefulList::new(), StatefulList::new(), StatefulList::new()],
            current_board: 0,
        }
    }
}



fn main() -> Result<(), Box<dyn Error>> {
    // Capture stdout and create a terminal
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new();
    let mut events = Events::new(); // create event listener


    // Check for updates && redraw terminal
    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .margin(1)
                .direction(Direction::Vertical)
                .constraints([ Constraint::Percentage(4), Constraint::Percentage(8), Constraint::Percentage(45), Constraint::Percentage(33), ].as_ref())
                .split(frame.size());

            let help_msg_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([ Constraint::Percentage(100) ].as_ref())
                .split(layout[0]);

            let kanban_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([ Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33), ].as_ref())
                .split(layout[2]);

            let ticket_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([ Constraint::Percentage(50), Constraint::Percentage(50), ].as_ref())
                .split(layout[3]);


            // Create Help message
            let (msg, style) = match app.input_mode {
                InputMode::Normal => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to exit, "),
                        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to start editing."),
                    ],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),

                InputMode::Input => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to stop editing, "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to record the message"),
                    ],
                    Style::default(),
                ),

                InputMode::Move => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("up / down / left / right", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to move around."),
                    ],
                    Style::default(),
                ),
            };

            let mut text = Text::from(Spans::from(msg));
            text.patch_style(style);
            let help_message = Paragraph::new(text);


            // Create WIDIGETS:
            //
            // INPUT WIDIGET
            let input_widget = Paragraph::new(app.input.as_ref())
                .block(Block::default().title("Input").borders(Borders::ALL));
            
            // We need to clone because we can't own it twice
            // is this now was Box or Rc is for??
            // TODOO LIST
            let todo_list = app.kanban[0].items.clone();
            let kanban_todo_items: Vec<ListItem> = 
                todo_list.iter().map(|ticket| {
                    let lines = vec![ Spans::from(ticket.title.as_ref()) ];
                    ListItem::new(lines)
                })
                .collect();
            let kanban_list_widget = List::new(kanban_todo_items)
                .block(Block::default().title("todo").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::LightBlue));

            // IN PROGRESS LIST
            let progress_list = app.kanban[1].items.clone();
            let kanban_progress_items: Vec<ListItem> = 
                progress_list.iter().map(|ticket| {
                    let lines = vec![ Spans::from(ticket.title.as_ref()) ];
                    ListItem::new(lines)
                })
                .collect();
            let kanban_progress_widget = List::new(kanban_progress_items)
                .block(Block::default().title("progress").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::LightGreen));

            // DONE LIST
            let done_list = app.kanban[2].items.clone();
            let kanban_done_items: Vec<ListItem> = 
                done_list.iter().map(|ticket| {
                    let lines = vec![ Spans::from(ticket.title.as_ref()) ];
                    ListItem::new(lines)
                })
                .collect();
            let kanban_done_widget = List::new(kanban_done_items)
                .block(Block::default().title("progress").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::LightGreen));

            
            // Place our WIDIGETS
            frame.render_widget(help_message, layout[0]);
            frame.render_widget(input_widget, layout[1]);
            frame.render_stateful_widget(
                kanban_list_widget, kanban_layout[0], &mut app.kanban[0].state);
            frame.render_stateful_widget(
                kanban_progress_widget, kanban_layout[1], &mut app.kanban[1].state);
            frame.render_stateful_widget(
                kanban_done_widget, kanban_layout[2], &mut app.kanban[2].state);


            // Set the CURSOR
            match app.input_mode {
                InputMode::Input => {
                    frame.set_cursor(
                        layout[0].x + app.input.len() as u16 + 1,
                        layout[0].y + 2,
                    )
                }
                _ => {}
            }
        // END terminal.draw()
        })?;

        // Handle keyboard EVENTS (inputs)
        if let Event::Input(input) = events.next()? {
            match app.input_mode {
                InputMode::Normal => match input {
                    Key::Char('i') => { app.input_mode = InputMode::Input; }
                    Key::Char('m') => { app.input_mode = InputMode::Move; }
                    Key::Char('q') => { break; }
                    _ => {}
                }
                InputMode::Input => match input {
                    Key::Char('\n') => {
                        let ticket_title = app.input.clone();
                        app.kanban[app.current_board as usize].items.push(Ticket::new(ticket_title));
                        app.kanban[app.current_board as usize].state.select(Some(0));
                        app.input = String::new();
                    }

                    Key::Char(key) => { app.input.push(key); }
                    Key::Backspace => { app.input.pop(); }
                    Key::Esc => { app.input_mode = InputMode::Normal; }
                    _ => { break; }

                }
                InputMode::Move => match input {
                    Key::Esc => { app.input_mode = InputMode::Normal }
                    Key::Down => { app.kanban[app.current_board as usize].next(); }
                    Key::Up => { app.kanban[app.current_board as usize].previous(); }
                    Key::Left => {}
                    Key::Right => {
                        // unselect current board
                        app.kanban[app.current_board as usize].unselect();
                        // if there is another board, move to it, or loop around
                        match app.current_board {
                            next_board => {
                                if next_board as usize >= app.kanban.len() - 1 {
                                    app.current_board = 0;
                                }
                                else{
                                    app.current_board = next_board + 1;
                                }
                            }
                        };
                        // select the first item on that boards list
                        app.kanban[app.current_board as usize]
                            .state.select(Some(0));
                    }
                    _ => {}
                }
            } 
        }
    }
    // End of program, Exit gracefully
    Ok(())
}
