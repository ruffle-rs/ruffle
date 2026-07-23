var sound = new Sound();
sound.onSoundComplete = function() {
    trace("Sound complete");
};
sound.onLoad = function(s) {
    trace("onLoad " + s);
    sound.start(0,9);
};

trace("loading sound");
sound.loadSound("http://localhost:8000/noise.mp3", true);
sound.stop();
trace("loading again");
sound.loadSound("http://localhost:8000/noise.mp3", true);
trace("after loading again");
