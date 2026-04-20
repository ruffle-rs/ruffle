var i = 0;

var sound = new Sound();
sound.onSoundComplete = function() {
    trace("Sound complete");
    if (i < 1) {
        ++i;
        sound.start();
    }
};

sound.loadSound("noise.mp3", false);
sound.setVolume(50);
sound.start();
sound.start();
