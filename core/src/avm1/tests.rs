use crate::avm1::error::Error;
use crate::avm1::test_utils::with_avm;
use crate::avm1::TObject;

#[test]
fn locals_into_form_values() {
    with_avm(19, |activation, context, _this| -> Result<(), Error> {
        let my_local_values = activation.avm().run_in_avm(
            context,
            19,
            *context.levels.get(&0).expect("_level0 in test"),
            |activation, context| {
                let my_locals = activation.activation().read().scope().locals().to_owned();
                my_locals
                    .set("value1", "string".into(), activation, context)
                    .unwrap();
                my_locals
                    .set("value2", 2.0.into(), activation, context)
                    .unwrap();
                activation.locals_into_form_values(context)
            },
        );

        assert_eq!(my_local_values.len(), 2);
        assert_eq!(my_local_values.get("value1"), Some(&"string".to_string()));
        assert_eq!(my_local_values.get("value2"), Some(&"2".to_string()));

        Ok(())
    });
}
