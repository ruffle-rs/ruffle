//! `flash.system.WorkerDomain` native methods

use crate::avm2::activation::Activation;
use crate::avm2::object::{WorkerDomainObject, WorkerObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2_stub_method;

/// Implements `WorkerDomain.createWorker`
pub fn create_worker<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.WorkerDomain", "createWorker");

    let _swf = args.get_object(activation, 0, "swf")?;
    let _give_app_privileges = args.get_bool(1);

    let worker = WorkerObject::new(activation);

    Ok(worker.into())
}

pub fn instantiate_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let worker_domain = WorkerDomainObject::new(activation);

    Ok(worker_domain.into())
}
