use egui::emath::RectTransform;
use egui::{pos2, Rect};
use egui_kittest::Harness;
use endgame_egui::{HollowArrowStyle, LabelStyle};

fn harness_painter(fnc: impl Fn(&egui::Painter) + 'static) -> Harness<'static> {
    Harness::new_ui(move |ui| {
        let painter = ui.painter_at(ui.max_rect());
        fnc(&painter);
    })
}

fn harness_transform_painter(fnc: impl Fn(&RectTransform, &egui::Painter) + 'static) -> Harness<'static> {
    Harness::new_ui(move |ui| {
        // TODO egui seems to blow up if we just used the identity transform?
        let rect_transform = RectTransform::from_to(
            Rect::from([pos2(400.0, 400.0), pos2(800.0, 800.0)]),
            Rect::from([pos2(0.0, 0.0), pos2(400.0, 400.0)]),
        );
        let painter = ui.painter_at(ui.max_rect());
        fnc(&rect_transform, &painter);
    })
}


#[test]
fn test_render_disallowed() {
    let mut harness = harness_painter(|painter| {
        endgame_egui::render_disallowed(
            egui::pos2(100.0, 100.0),
            50.0,
            5.0,
            &painter,
        );
    });

    harness.run();
}

#[test]
fn test_render_arrow() {
    let mut harness = harness_painter(|painter| {
        let style = endgame_egui::SolidArrowStyle {
            color: egui::Color32::GREEN,
            width: 2.0,
            to_head: true,
            from_head: false,
            label: Some(LabelStyle {
                color: egui::Color32::BLACK,
                font_size: 14.0,
                add_shadow: Some(egui::Color32::LIGHT_GRAY),
            }),
        };
        endgame_egui::render_arrow(
            pos2(100.0, 100.0),
            pos2(200.0, 200.0),
            &style,
            Some("Arrow"),
            &painter,
        );
    });

    harness.run();
}

#[test]
fn test_render_hollow_arrow() {
    let mut harness = harness_painter(|painter| {
        let style = HollowArrowStyle {
            fill_color: egui::Color32::BLUE,
            border_color: egui::Color32::BLACK,
            width: 2.0,
            label: Some(LabelStyle {
                color: egui::Color32::BLACK,
                font_size: 14.0,
                add_shadow: Some(egui::Color32::LIGHT_GRAY),
            }),
        };
        endgame_egui::render_hollow_arrow(
            pos2(300.0, 100.0),
            pos2(200.0, 200.0),
            &style,
            Some("Hollow Arrow"),
            &painter,
        );
    });

    harness.run();
}

#[test]
fn test_render_hollow_self_arrow() {
    let mut harness = harness_painter(|painter| {
        let style = HollowArrowStyle {
            fill_color: egui::Color32::BLUE,
            border_color: egui::Color32::BLACK,
            width: 2.0,
            label: Some(LabelStyle {
                color: egui::Color32::BLACK,
                font_size: 14.0,
                add_shadow: Some(egui::Color32::LIGHT_GRAY),
            }),
        };
        endgame_egui::render_hollow_self_arrow(
            pos2(300.0, 300.0),
            &style,
            Some("Self Arrow"),
            &painter,
        );
    });

    harness.run();
}