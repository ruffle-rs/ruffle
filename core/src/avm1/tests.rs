use crate::avm1::error::Error;
use crate::avm1::test_utils::with_avm;
use crate::avm1::TObject;

#[test]
fn locals_into_form_values() {
    with_avm(19, |activation, _this| -> Result<(), Error> {
        let my_locals = activation.scope().locals().to_owned();
        my_locals
            .set("value1", "string".into(), activation)
            .unwrap();
        my_locals.set("value2", 2.into(), activation).unwrap();
        let my_local_values = activation.locals_into_form_values();

        assert_eq!(my_local_values.len(), 2);
        assert_eq!(my_local_values.get("value1"), Some(&"string".to_string()));
        assert_eq!(my_local_values.get("value2"), Some(&"2".to_string()));
        assert_eq!(
            my_local_values.keys().cloned().collect::<Vec<String>>(),
            vec!["value2".to_string(), "value1".to_string()]
        );

        Ok(())
    });
}
