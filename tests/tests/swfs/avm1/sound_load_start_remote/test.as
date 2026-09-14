var i = 0;

var sound = new Sound();
sound.onSoundComplete = function() {
    trace("Sound complete");
    if (i < 1) {
        ++i;
        sound.start();
    }
};
sound.onLoad = function(s) {
    trace("onLoad " + s);
};

sound.loadSound("http://localhost:8000/noise.mp3");
sound.setVolume(50);
sound.start();
sound.start();
