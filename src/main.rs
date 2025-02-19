use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use std::{error::Error, io, fs};
use kube::config::Kubeconfig;

#[derive(Clone)]
struct KubeContext {
    name: String,
    cluster: Option<String>,
    user: Option<String>,
}

struct App {
    contexts: Vec<KubeContext>,
    current_context: String,
    selected_index: usize,
    config_path: String,
}

impl App {
    fn new() -> Result<Self, Box<dyn Error>> {
        let config_path = std::env::var("KUBECONFIG").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/.kube/config", home)
        });

        let (contexts, current_context) = get_kubernetes_contexts(&config_path)?;
        
        Ok(App {
            contexts,
            current_context,
            selected_index: 0,
            config_path,
        })
    }

    fn next(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.contexts.len();
    }

    fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.contexts.len() - 1;
        }
    }
}

fn get_kubernetes_contexts(config_path: &str) -> Result<(Vec<KubeContext>, String), Box<dyn Error>> {
    let kube_config = fs::read_to_string(config_path)?;
    let config: Kubeconfig = serde_yaml::from_str(&kube_config)?;
    
    let current_context = config.current_context.unwrap_or_default();
    
    let contexts: Vec<KubeContext> = config.contexts.iter().map(|ctx| {
        KubeContext {
            name: ctx.name.clone(),
            cluster: ctx.context.as_ref().map(|c| c.cluster.clone()),
            user: ctx.context.as_ref().and_then(|c| c.user.clone()),
        }
    }).collect();

    Ok((contexts, current_context))
}

fn set_kubernetes_context(config_path: &str, context_name: &str) -> Result<(), Box<dyn Error>> {
    let kube_config = fs::read_to_string(config_path)?;
    let mut config: Kubeconfig = serde_yaml::from_str(&kube_config)?;
    
    config.current_context = Some(context_name.to_string());
    
    let new_config = serde_yaml::to_string(&config)?;
    fs::write(config_path, new_config)?;
    
    Ok(())
}

fn run_app() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;

    loop {
        terminal.draw(|f| {
            let size = f.area();
            
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Min(0),     // Context list with headers
                ].as_ref())
                .split(size);

            // Create header as the first list item
            let header = ListItem::new(
                format!(
                    "    {:<30} │ {:<30} │ {:<30}",
                    "CONTEXT", "CLUSTER", "USER"
                )
            ).style(Style::default().add_modifier(Modifier::BOLD));

            // Create the context items
            let mut items = vec![header];
            items.extend(app
                .contexts
                .iter()
                .map(|context| {
                    let current_marker = if context.name == app.current_context { "(*)" } else { "   " };
                    let display_text = format!(
                        "{} {:<30} │ {:<30} │ {:<30}",
                        current_marker,
                        truncate_str(&context.name, 30),
                        truncate_str(context.cluster.as_deref().unwrap_or("-"), 30),
                        truncate_str(context.user.as_deref().unwrap_or("-"), 30)
                    );
                    ListItem::new(display_text)
                }));

            let list = List::new(items)
                .block(Block::default()
                    .title("Kubernetes Contexts (Press 'q' to quit, ↑/↓ or j/k to navigate, Enter to select)")
                    .borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol("► ");

            f.render_stateful_widget(
                list, 
                chunks[0], 
                &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_index.saturating_add(1)))
            );
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('j') | KeyCode::Down => app.next(),
                KeyCode::Char('k') | KeyCode::Up => app.previous(),
                KeyCode::Enter => {
                    let selected_context = &app.contexts[app.selected_index];
                    set_kubernetes_context(&app.config_path, &selected_context.name)?;
                    app.current_context = selected_context.name.clone();
                    break;
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    run_app()?;
    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}