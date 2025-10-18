use egui::accesskit::Toggled;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
// TODO Rename due to redundancy?
use grid_demo::grid_demo::GridDemo;

#[test]
fn test_demo() {
    let mut harness = Harness::<GridDemo>::new_state(|ctx, state| {
        state.run(ctx);
    }, GridDemo::default());

    harness.run();

    let kinds = ["Square", "Hex", "Triangle"];
    let examples = GridDemo::examples();
    for kind in kinds {
        {
            let kind_radio = harness.get_by_label(kind);
            kind_radio.click();
            assert_eq!(kind_radio.accesskit_node().toggled(), Some(Toggled::True));
        }
        harness.run();
        for example in &examples {
            let example_ref = example.borrow();
            let label = example_ref.label();
            let example_radio = harness.get_by_label(label);
            example_radio.click();

            harness.run();
            // TODO Need to filter by the examples supported, by the grid kind.
            //assert_eq!(example_radio.accesskit_node().toggled(), Some(Toggled::True));


            //           let grid_view = harness.get_by_label("central_panel");
        }
    }
}