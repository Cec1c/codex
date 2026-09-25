use super::*;
use crate::ccu_welcome::WelcomeCard;
use pretty_assertions::assert_eq;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

#[test]
fn claude_welcome_and_composer_responsive() {
    let theme = parse_theme(
        include_str!("../assets/ccu-claude-theme.json"),
        "rainbow_color",
        1,
    )
    .expect("theme");
    let card = WelcomeCard {
        version: "0.157.0",
        model: "gpt-6-astra",
        reasoning: Some("high"),
        directory: "D:\\工作区\\codex-cli-ultra",
        yolo_mode: true,
        fast: false,
    };
    for (width, height) in [(80, 24), (120, 40)] {
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        let lines = card.lines(width, &theme);
        assert!(lines.iter().all(|line| line.width() <= usize::from(width)));
        let header_height = lines.len() as u16;
        Paragraph::new(lines).render(area, &mut buffer);
        let composer = Rect::new(0, header_height + 3, width, 3);
        crate::ccu_welcome::render_composer_rules(composer, &mut buffer, &theme);
        Line::from("❯ ").render(Rect::new(0, composer.y + 1, width, 1), &mut buffer);
        let progress = Line::from(theme.progress_spans(6));
        progress.render(Rect::new(2, composer.bottom(), width - 2, 1), &mut buffer);
        insta::assert_snapshot!(
            format!("claude_layout_{width}x{height}"),
            format!("{buffer:?}")
        );
        // Optional local preview contains only this deterministic test fixture.
        if let Some(directory) = std::env::var_os("CCU_PREVIEW_DIR") {
            let cells = buffer.content.iter().map(|cell| serde_json::json!({
                "text": cell.symbol(), "fg": format!("{:?}", cell.fg), "bg": format!("{:?}", cell.bg)
            })).collect::<Vec<_>>();
            std::fs::write(
                PathBuf::from(directory).join(format!("claude-{width}x{height}.json")),
                serde_json::to_vec(
                    &serde_json::json!({"width":width,"height":height,"cells":cells}),
                )
                .unwrap(),
            )
            .unwrap();
        }
    }
    for width in [0, 1, 5, 6, 20, 79, 99, 100, 180] {
        assert!(
            card.lines(width, &theme)
                .iter()
                .all(|line| line.width() <= usize::from(width))
        );
    }
}

#[test]
fn claude_progress_color_boundaries_and_clamping() {
    let theme = parse_theme(
        include_str!("../assets/ccu-claude-theme.json"),
        "rainbow_color",
        1,
    )
    .expect("theme");
    let lines = [0, 6, 50, 100, 255]
        .into_iter()
        .map(|percent| Line::from(theme.progress_spans(percent)))
        .collect::<Vec<_>>();
    assert_eq!(lines.last().unwrap().to_string(), "[██████████] 100%");
    let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 5));
    Paragraph::new(lines).render(buffer.area, &mut buffer);
    insta::assert_snapshot!(format!("{buffer:?}"));
}
