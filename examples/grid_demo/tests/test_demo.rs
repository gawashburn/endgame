#![feature(hash_map_macro)]

use egui::accesskit::Toggled;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
// TODO Rename due to redundancy?
use grid_demo::grid_demo::GridDemo;

#[test]
fn test_demo() {
    let mut harness = Harness::<GridDemo>::new_state(
        |ctx, state| {
            state.run(ctx);
        },
        GridDemo::default(),
    );

    harness.run();

    use endgame_grid::dynamic::Kind::*;
    let kind_map = hash_map! { Square => "Square", Hex => "Hex", Triangle => "Triangle" };
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
                // TODO Add testing around interacting with displayed grid.
                //let grid_view = harness.get_by_label("central_panel");
            }
        }
    }
}
