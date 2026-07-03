use egui::accesskit::Toggled;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use grid_demo::app::GridDemo;
use std::collections::HashMap;

#[test]
fn test_demo() {
    let mut harness = Harness::<GridDemo>::new_ui_state(
        |ui, state| {
            state.run(ui);
        },
        GridDemo::default(),
    );

    harness.run();

    // Iterate through all grid kinds, then all examples that support the given
    // kind.
    use endgame_grid::dynamic::Kind::*;
    let kind_map = HashMap::from([(Square, "Square"), (Hex, "Hex"), (Triangle, "Triangle")]);
    let examples = GridDemo::examples();
    for (kind, kind_label) in kind_map {
        // TODO Write helpers for interacting, running and then checking.
        {
            let kind_radio = harness.get_by_label(kind_label);
            kind_radio.click();
        }
        harness.run();
        {
            let kind_radio = harness.get_by_label(kind_label);
            assert_eq!(
                kind_radio.accesskit_node().toggled(),
                Some(Toggled::True),
                "Grid kind not toggled for {}",
                kind
            );
        }
        for example in examples
            .iter()
            .filter(|e| e.borrow().supports_grid_kind(kind))
        {
            let example_ref = example.borrow();
            let label = example_ref.label();
            {
                let example_radio = harness.get_by_label(label);
                example_radio.click();
            }
            harness.run();
            {
                let example_radio = harness.get_by_label(label);
                assert_eq!(
                    example_radio.accesskit_node().toggled(),
                    Some(Toggled::True)
                );
                harness.drag_at(egui::pos2(320.0, 240.0));
            }
            harness.run();
            {
                harness.drop_at(egui::pos2(320.0, 240.0));
            }
            harness.run();

            // TODO More example specific testing.
        }
    }
}
