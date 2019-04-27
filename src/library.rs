use crate::character::Character;
use crate::display_object::DisplayObjectNode;
use crate::graphic::Graphic;
use crate::movie_clip::MovieClip;
use std::collections::HashMap;
use swf::CharacterId;

pub struct Library {
    characters: HashMap<CharacterId, Character>,
}

impl Library {
    pub fn new() -> Library {
        Library {
            characters: HashMap::new(),
        }
    }

    pub fn register_character(&mut self, id: CharacterId, character: Character) {
        // TODO(Herschel): What is the behavior if id already exists?
        self.characters.insert(id, character);
    }

    pub fn contains_character(&self, id: CharacterId) -> bool {
        self.characters.contains_key(&id)
    }

    pub fn instantiate_display_object(
        &self,
        id: CharacterId,
    ) -> Result<DisplayObjectNode, Box<std::error::Error>> {
        match self.characters.get(&id) {
            Some(Character::Graphic {
                //image,
                x_min,
                y_min,
            }) => Ok(DisplayObjectNode::Graphic(Graphic::new(
                //image.clone(),
                *x_min, *y_min,
            ))),
            Some(Character::MovieClip {
                tag_stream_start,
                num_frames,
            }) => Ok(DisplayObjectNode::MovieClip(MovieClip::new_with_data(
                *tag_stream_start,
                *num_frames,
            ))),
            Some(_) => Err("Not a DisplayObject".into()),
            None => Err("Character id doesn't exist".into()),
        }
    }
}
