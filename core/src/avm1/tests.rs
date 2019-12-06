use crate::avm1::activation::Activation;
use crate::avm1::test_utils::with_avm;
use crate::avm1::TObject;
use gc_arena::GcCell;

#[test]
fn locals_into_form_values() {
    with_avm(19, |avm, context, _this| {
        let my_activation =
            Activation::from_nothing(19, avm.global_object_cell(), context.gc_context);
        let my_locals = my_activation.scope().locals().to_owned();

        my_locals
            .set("value1", "string".into(), avm, context, my_locals)
            .unwrap();
        my_locals
            .set("value2", 2.0.into(), avm, context, my_locals)
            .unwrap();

        avm.insert_stack_frame(GcCell::allocate(context.gc_context, my_activation));

        let my_local_values = avm.locals_into_form_values(context);

        assert_eq!(my_local_values.len(), 2);
        assert_eq!(my_local_values.get("value1"), Some(&"string".to_string()));
        assert_eq!(my_local_values.get("value2"), Some(&"2".to_string()));
    });
}
