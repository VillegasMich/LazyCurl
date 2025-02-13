use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use reqwest::blocking::Client;
use std::{collections::HashMap, io};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    let mut selected_method = 0;
    let headers: HashMap<String, String> = HashMap::new();
    let params: HashMap<String, String> = HashMap::new();
    let body = String::new();
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
    let client = Client::new();
    let mut response_text = String::from("Response will appear here...");
    let mut options_mode = 0; // 0: Headers, 1: Body, 2: Params

    loop {
        terminal.draw(|frame| {
            let size = frame.area();

            // Split the UI into left (methods) and right (rest of UI)
            let main_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(10), // Left panel for HTTP methods
                    Constraint::Percentage(90), // Right panel for input, response, etc.
                ])
                .split(size);

            // Further split the right panel into vertical sections
            let right_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10), // Header
                    Constraint::Percentage(10), // URL Input
                    Constraint::Percentage(40), // Additional UI (Future feature)
                    Constraint::Percentage(40), // Response box
                ])
                .split(main_layout[1]);

            // Header
            let header = Paragraph::new(Line::from(vec![Span::styled(
                "LazyCurl - HTTP Requester",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

            // URL Input Box
            let input_box = Paragraph::new(input.clone())
                .block(Block::default().title("Enter URL").borders(Borders::ALL))
                .alignment(Alignment::Center);

            // Method Selector List
            let methods_items: Vec<ListItem> = methods
                .iter()
                .enumerate()
                .map(|(i, &method)| {
                    let style = if i == selected_method {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Span::styled(method, style))
                })
                .collect();

            let method_box = List::new(methods_items)
                .block(Block::default().title("HTTP Method").borders(Borders::ALL));

            let options_text = if options_mode == 0 {
                format!("Headers: {:?}", headers)
            } else if options_mode == 1 {
                format!("Body: {}", body)
            } else {
                format!("Params: {:?}", params)
            };
            let options_box = Paragraph::new(options_text).block(
                Block::default()
                    .title("Options (H: Headers, B: Body, P: Params)")
                    .borders(Borders::ALL),
            );

            // Response Box
            let response_box = Paragraph::new(response_text.clone())
                .block(Block::default().title("Response").borders(Borders::ALL));

            // Render UI Components
            frame.render_widget(method_box, main_layout[0]); // Left panel (Method selector)
            frame.render_widget(header, right_layout[0]); // Header (Right panel)
            frame.render_widget(input_box, right_layout[1]); // Input field (Right panel)
            frame.render_widget(options_box, right_layout[2]); // Input field (Right panel)
            frame.render_widget(response_box, right_layout[3]); // Response box (Right panel)
        })?;

        // Event handling
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Up => {
                        if selected_method > 0 {
                            selected_method -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected_method < methods.len() - 1 {
                            selected_method += 1;
                        }
                    }
                    KeyCode::Char('H') => options_mode = 0,
                    KeyCode::Char('B') => options_mode = 1,
                    KeyCode::Char('P') => options_mode = 2,
                    KeyCode::Enter => {
                        if !input.is_empty() {
                            let method = methods[selected_method];
                            let url = input.clone();
                            response_text = make_request(&client, method, &url, &headers, &body);
                        }
                    }
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

/// Handles making an HTTP request based on user selection
fn make_request(
    client: &Client,
    method: &str,
    url: &str,
    headers: &HashMap<String, String>,
    body: &str,
) -> String {
    let mut request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => return "Invalid Method".to_string(),
    };

    for (key, value) in headers {
        request = request.header(key, value);
    }

    if method != "GET" {
        request = request.body(body.to_string());
    }

    request
        .send()
        .and_then(|res| res.text())
        .unwrap_or_else(|_| "Failed to make request".into())
}
