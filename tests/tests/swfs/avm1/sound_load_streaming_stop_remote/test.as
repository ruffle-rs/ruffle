var sound = new Sound();
sound.onSoundComplete = function() {
    trace("Sound complete");
};
sound.onLoad = function(s) {
    trace("onLoad " + s);
};

trace("before");
sound.loadSound("http://localhost:8000/noise.mp3", true);
trace("after");
sound.stop();
