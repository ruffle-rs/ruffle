use std::collections::HashMap;
use std::io::Write;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use zip::write::SimpleFileOptions;

#[wasm_bindgen]
#[derive(Default)]
pub struct ZipWriter {
    files: HashMap<String, Vec<u8>>,
}

#[wasm_bindgen]
impl ZipWriter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(js_name = "addFile")]
    pub fn add_file(&mut self, name: String, bytes: Vec<u8>) {
        self.files.insert(name, bytes);
    }

    pub fn save(&self) -> Result<Vec<u8>, JsValue> {
        let mut buffer = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
        for (name, content) in &self.files {
            zip.start_file(name.to_string(), SimpleFileOptions::default())
                .map_err(|e| e.to_string())?;
            zip.write_all(content).map_err(|e| e.to_string())?;
        }

        zip.finish().map_err(|e| e.to_string())?;
        Ok(buffer)
    }
}
