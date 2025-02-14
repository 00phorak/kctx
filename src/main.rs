use std::io::{stdout, Result};

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::Constraint,
    prelude::CrosstermBackend,
    style::Stylize,
    text::Line,
    widgets::{Cell, Row, Table, TableState},
    Terminal,
};

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // let mut contexts: Vec<Line> = vec![];
    let config = kube::config::Kubeconfig::read().unwrap();
    let current_context = match &config.current_context {
        Some(cc) => String::from(cc),
        None => String::from(""),
    };
    let header = Row::new(vec![
        Cell::from("Current Context").bold(),
        Cell::from("Name").bold(),
        Cell::from("Cluster").bold(),
        Cell::from("Username").bold(),
    ]);
    let widths = [
        Constraint::Percentage(5),
        Constraint::Percentage(35),
        Constraint::Percentage(35),
        Constraint::Percentage(25),
    ];
    // block
    let rows: Vec<Row> = config
        .contexts
        .iter()
        .map(|x| {
            let x = x.to_owned();
            let context = x.context.unwrap();
            let current = if x.name == current_context {
                String::from("*")
            } else {
                String::from("")
            };
            let line = Row::new(vec![
                current,
                String::from(x.name),
                String::from(context.cluster),
                String::from(context.user.unwrap()),
            ]);
            line
        })
        .collect();

    let table = Table::new(rows, widths).header(header).column_spacing(1);
    let _line = Line::from(vec!["hello".red(), " ".into(), "world".red().bold()]);
    let mut table_state = TableState::default();
    table_state.select(Some(3)); // select the forth row (0-indexed)

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_stateful_widget(
                // Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                // Paragraph::new(contexts.clone())
                //     .block(
                //         Block::new()
                //             .title("Choose K8s Context")
                //             .borders(Borders::ALL),
                //     )
                //     .alignment(ratatui::prelude::Alignment::Center)
                //     .wrap(Wrap { trim: true }),
                table.clone(),
                area,
                &mut table_state,
            );
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
