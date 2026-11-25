// Plays a sound depending on the level: one targeting the level, one without
// a target, and a streamed one.
var name = String(this);
var sound;
if (name == "_level1") {
    sound = new Sound(this);
    sound.onLoad = function() {
        sound.start();
    };
    sound.loadSound("tone.mp3", false);
} else if (name == "_level2") {
    sound = new Sound();
    sound.onLoad = function() {
        sound.start();
    };
    sound.loadSound("tone.mp3", false);
} else {
    sound = new Sound(this);
    sound.loadSound("tone.mp3", true);
}
sound.onSoundComplete = function() {
    trace("onSoundComplete: sound of " + name);
};
_level0.childLoaded(name, sound);
