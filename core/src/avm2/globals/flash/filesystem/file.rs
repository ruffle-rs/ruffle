use crate::avm2::object::FileReferenceObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::avm2::{ArrayObject, ArrayStorage};
use crate::string::AvmString;
use std::path::Path;

use crate::avm2::object::FileReference;

use std::fs;
use url::Url;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    // TODO: Implement the complete path syntax.
    if let Value::String(path) = args[0] {
        this.init_from_path_buf(if &path == b"app:" {
            match Url::parse(activation.context.swf.url()) {
                Ok(url) if url.scheme() == "file" => {
                    // Remove .swf from path
                    Path::new(url.path()).parent().unwrap().into()
                }
                _ => {
                    tracing::warn!("File::init: File.applicationDirectory could not be resolved");
                    "/tmp/app".into()
                }
            }
        } else if &path == b"app-storage:" {
            "/tmp/app-storage".into() // XXXX
        } else {
            let path = path.to_string();
            Path::new(&path).to_path_buf()
        });
    }

    Ok(Value::Undefined)
}

pub fn get_exists<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let exists = match *this.file_reference() {
        FileReference::FilePath(ref path) => path.exists(),
        _ => panic!("wrong class"),
    };
    Ok(exists.into())
}

pub fn get_is_directory<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let is_dir = match *this.file_reference() {
        FileReference::FilePath(ref path) => path.is_dir(),
        _ => panic!("wrong class"),
    };
    Ok(is_dir.into())
}

pub fn get_native_path<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let native_path = match *this.file_reference() {
        FileReference::FilePath(ref path) => path.to_str().unwrap().to_owned(),
        _ => panic!("wrong class"),
    };
    Ok(AvmString::new_utf8(activation.gc(), &native_path).into())
}

pub fn get_url<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let url = match *this.file_reference() {
        FileReference::FilePath(ref path) => format!("file://{}", path.to_str().unwrap()),
        _ => panic!("wrong class"),
    };
    Ok(AvmString::new_utf8(activation.gc(), &url).into())
}

pub fn create_directory<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let path = match *this.file_reference() {
        FileReference::FilePath(ref path) => path.clone(),
        _ => panic!("wrong class"),
    };
    fs::create_dir_all(path).expect("create_dir_all failed!");
    Ok(Value::Undefined)
}

fn listing<'gc>(activation: &mut Activation<'_, 'gc>, path: &Path) -> Value<'gc> {
    let mut arr = vec![];

    for entry in fs::read_dir(path).unwrap() {
        let new_file = FileReferenceObject::new_file(activation, entry.unwrap().path()).unwrap();
        arr.push(Some(new_file.into()));
    }

    let storage = ArrayStorage::from_storage(arr);
    let array = ArrayObject::from_storage(activation, storage).unwrap();
    array.into()
}

pub fn get_directory_listing<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let path = match *this.file_reference() {
        FileReference::FilePath(ref path) => path.clone(),
        _ => panic!("wrong class"),
    };
    Ok(listing(activation, &path))
}

pub fn resolve_path<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_file_reference().unwrap();

    let path = args.get_string(activation, 0)?;
    let path_str: String = path.to_string();

    let resolved_path = match *this.file_reference() {
        FileReference::FilePath(ref path_buf) => path_buf.join(path_str),
        _ => panic!("wrong class"),
    };
    Ok(FileReferenceObject::new_file(activation, resolved_path)?.into())
}
