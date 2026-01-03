use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};

use crate::app::App;
use crate::logo;
use crate::system_info::format_bytes;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    // 检查终端尺寸，如果太小则使用简化布局
    if size.height < 20 || size.width < 60 {
        draw_compact_layout(f, app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // for drawing title
            Constraint::Min(0),    // for drawing infos
            Constraint::Length(3), // for drawing help
        ])
        .split(size);

    draw_title(f, chunks[0]);

    // 根据宽度调整布局
    if size.width < 100 {
        // 窄屏幕时使用垂直布局
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // ASCII art
                Constraint::Min(0),    // system information
            ])
            .split(chunks[1]);

        draw_ascii_art(f, main_chunks[0]);
        draw_all_info_vertical(f, main_chunks[1], app);
    } else {
        // 宽屏幕时使用水平布局
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // left-side: ASCII art
                Constraint::Percentage(60), // right-side: system information
            ])
            .split(chunks[1]);

        draw_ascii_art(f, main_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // system information
                Constraint::Percentage(70), // hardware information
            ])
            .split(main_chunks[1]);

        draw_system_info(f, right_chunks[0], app);
        draw_hardware_info(f, right_chunks[1], app);
    }

    draw_help(f, chunks[2]);
}

fn draw_title(f: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new("🦀 sysfetch-rs")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(title, area);
}

fn draw_ascii_art(f: &mut Frame, area: ratatui::layout::Rect) {
    let ascii_art = logo::get_logo();

    let paragraph = Paragraph::new(ascii_art)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("🖥️  System")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(paragraph, area);
}

fn draw_system_info(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let info = &app.system_info;

    let text = vec![
        Line::from(vec![
            Span::styled(
                "OS: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{} {}", info.os_name, info.os_version)),
        ]),
        Line::from(vec![
            Span::styled(
                "Kernel: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.kernel_version),
        ]),
        Line::from(vec![
            Span::styled(
                "Host: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.hostname),
        ]),
        Line::from(vec![
            Span::styled(
                "User: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.username),
        ]),
        Line::from(vec![
            Span::styled(
                "Uptime: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.uptime),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("📋 System Info")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_hardware_info(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let info = &app.system_info;

    // 动态计算约束，避免内容被隐藏
    let available_height = area.height.saturating_sub(2); // 减去边框
    let min_section_height = 3;
    let sections = 5; // CPU, Memory, GPU, IP, Disk

    let constraints = if available_height >= sections * min_section_height {
        // 有足够空间时使用固定高度
        vec![
            Constraint::Length(4), // CPU information
            Constraint::Length(4), // Memory information
            Constraint::Length(3), // GPU information
            Constraint::Length(3), // IP information
            Constraint::Min(3),    // Disk information
        ]
    } else {
        // 空间不足时使用百分比分配
        vec![
            Constraint::Percentage(20), // CPU information
            Constraint::Percentage(25), // Memory information
            Constraint::Percentage(15), // GPU information
            Constraint::Percentage(15), // IP information
            Constraint::Percentage(25), // Disk information
        ]
    };

    // Create vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }));

    // CPU information
    draw_cpu_info(f, chunks[0], info);

    // Memory information
    draw_memory_info(f, chunks[1], info);

    // GPU information
    draw_gpu_info(f, chunks[2], info);

    // IP information
    draw_ip_info(f, chunks[3], info);

    // Disk information
    draw_disk_info(f, chunks[4], info);

    // Draw outer border
    let block = Block::default()
        .title("⚙️  Hardware Info")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(block, area);
}

/// Draw CPU information
fn draw_cpu_info(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    info: &crate::system_info::SystemInfo,
) {
    let text = vec![Line::from(vec![
        Span::styled(
            "CPU: ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{} ({} Cores)", info.cpu_model, info.cpu_cores)),
    ])];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title("🔥 CPU")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );

    f.render_widget(paragraph, area);
}

/// Draw memory information
fn draw_memory_info(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    info: &crate::system_info::SystemInfo,
) {
    let memory_percent = if info.memory_total > 0 {
        (info.memory_used as f64 / info.memory_total as f64 * 100.0) as u16
    } else {
        0
    };

    let available_height = area.height.saturating_sub(2);

    if available_height >= 3 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Memory usage text
                Constraint::Min(1),    // Progress bar
            ])
            .split(area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }));

        // Memory usage text
        let memory_text = Paragraph::new(format!(
            "{} / {} ({}%)",
            format_bytes(info.memory_used),
            format_bytes(info.memory_total),
            memory_percent
        ));
        f.render_widget(memory_text, chunks[0]);

        // Memory usage progress bar
        let memory_gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(Color::Blue))
            .percent(memory_percent)
            .label("")
            .use_unicode(true);
        f.render_widget(memory_gauge, chunks[1]);
    } else {
        // 空间不足时只显示文本
        let memory_text = Paragraph::new(format!(
            "{} / {} ({}%)",
            format_bytes(info.memory_used),
            format_bytes(info.memory_total),
            memory_percent
        ))
        .alignment(Alignment::Center);
        f.render_widget(
            memory_text,
            area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }),
        );
    }

    // Draw outer border
    let block = Block::default()
        .title("💾 Memory")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(block, area);
}

/// Draw GPU information
fn draw_gpu_info(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    info: &crate::system_info::SystemInfo,
) {
    let text = vec![Line::from(vec![
        Span::styled(
            "GPU: ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&info.gpu_info),
    ])];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("🎮 GPU")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw IP information
fn draw_ip_info(f: &mut Frame, area: ratatui::layout::Rect, info: &crate::system_info::SystemInfo) {
    let text = vec![Line::from(vec![
        Span::styled(
            "Local IP: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&info.local_ip),
    ])];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title("🌐 Network")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(paragraph, area);
}

/// Draw disk information
fn draw_disk_info(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    info: &crate::system_info::SystemInfo,
) {
    let disk_percent = if info.disk_total > 0 {
        (info.disk_used as f64 / info.disk_total as f64 * 100.0) as u16
    } else {
        0
    };

    // 根据可用高度调整布局
    let available_height = area.height.saturating_sub(2); // 减去边框

    if available_height >= 3 {
        // 有足够空间显示进度条
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Disk usage text
                Constraint::Min(1),    // Progress bar
            ])
            .split(area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }));

        // Disk usage text
        let disk_text = Paragraph::new(format!(
            "{} / {} ({}%)",
            format_bytes(info.disk_used),
            format_bytes(info.disk_total),
            disk_percent
        ));
        f.render_widget(disk_text, chunks[0]);

        // Disk usage progress bar
        let disk_gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(disk_percent)
            .label("")
            .use_unicode(true);
        f.render_widget(disk_gauge, chunks[1]);
    } else {
        // 空间不足时只显示文本
        let disk_text = Paragraph::new(format!(
            "{} / {} ({}%)",
            format_bytes(info.disk_used),
            format_bytes(info.disk_total),
            disk_percent
        ))
        .alignment(Alignment::Center);
        f.render_widget(
            disk_text,
            area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }),
        );
    }

    // Draw outer border
    let block = Block::default()
        .title("💿 Disk")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, area);
}

/// Draw help information
fn draw_help(f: &mut Frame, area: ratatui::layout::Rect) {
    let help_text = Paragraph::new("Press 'q' or 'Esc' to quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        );
    f.render_widget(help_text, area);
}

/// 紧凑布局，用于非常小的终端窗口
fn draw_compact_layout(f: &mut Frame, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // 简化标题
            Constraint::Min(0),    // 主要内容
            Constraint::Length(1), // 简化帮助
        ])
        .split(size);

    // 简化标题
    let title = Paragraph::new("sysfetch-rs")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // 紧凑的系统信息
    draw_compact_info(f, chunks[1], app);

    // 简化帮助
    let help = Paragraph::new("q: quit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

/// 垂直布局，用于窄屏幕
fn draw_all_info_vertical(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // 系统信息
            Constraint::Min(0),    // 硬件信息
        ])
        .split(area);

    draw_system_info(f, chunks[0], app);
    draw_hardware_info(f, chunks[1], app);
}

/// 紧凑的信息显示
fn draw_compact_info(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let info = &app.system_info;

    let text = vec![
        Line::from(vec![
            Span::styled(
                "OS: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{} {}", info.os_name, info.os_version)),
        ]),
        Line::from(vec![
            Span::styled(
                "Host: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.hostname),
        ]),
        Line::from(vec![
            Span::styled(
                "CPU: ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(&info.cpu_model),
        ]),
        Line::from(vec![
            Span::styled(
                "Memory: ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{} / {}",
                format_bytes(info.memory_used),
                format_bytes(info.memory_total)
            )),
        ]),
        Line::from(vec![
            Span::styled(
                "Disk: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{} / {}",
                format_bytes(info.disk_used),
                format_bytes(info.disk_total)
            )),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("System Info")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
