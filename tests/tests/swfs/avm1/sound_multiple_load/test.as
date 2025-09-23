var sound = new Sound();
sound.onSoundComplete = function() {
    trace("Sound complete");
};

sound.setVolume(50);
sound.loadSound("noise.mp3", false);
sound.start();

sound.loadSound("sound.mp3", false);
sound.start();
