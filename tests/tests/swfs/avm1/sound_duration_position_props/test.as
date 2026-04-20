function printProps(sound) {
    trace("  sound.getDuration = " + sound.getDuration);
    trace("  sound.getDuration() = " + sound.getDuration());
    trace("  sound.getPosition = " + sound.getPosition);
    trace("  sound.getPosition() = " + sound.getPosition());
    trace("  sound.duration = " + sound.duration);
    trace("  typeof sound.duration = " + (typeof sound.duration));
    trace("  sound.position = " + sound.position);
    trace("  typeof sound.position = " + (typeof sound.position));
}

function testSound(place, cb) {
    trace("// Place " + place);
    var sound = new Sound();
    if (place === -2) {
        trace("// Setting only duration");
        sound.duration = "x";
    }
    if (place === -1) {
        trace("// Setting only position");
        sound.position = "x";
    }
    if (place === 0) {
        trace("// Setting both");
        sound.position = "x";
        sound.duration = "x";
    }
    sound.onID3 = function(a, b) {
        trace("  onID3 called");
        printProps(sound);
        if (place === 1) {
            sound.position = "x";
            sound.duration = "x";
        }
    };
    sound.onLoad = function(a, b) {
        trace("  onLoad called");
        printProps(sound);
        if (place === 2) {
            sound.position = "x";
            sound.duration = "x";
            trace("  after set");
            printProps(sound);
        }
    };
    sound.onSoundComplete = function(a, b) {
        trace("  onSoundComplete called");
        printProps(sound);
        if (place === 3) {
            sound.position = "x";
            sound.duration = "x";
            trace("  after set");
            printProps(sound);
        }
        trace("  stopping");
        sound.stop();
        printProps(sound);
        if (place === 4) {
            sound.position = "x";
            sound.duration = "x";
            trace("  after set");
            printProps(sound);
        }
        cb();
    };
    sound.loadSound("sound.mp3", true);
}

testSound(0, function() {
    testSound(1, function() {
        testSound(2, function() {
            testSound(3, function() {
                testSound(4, function() {
                    testSound(-1, function() {
                        testSound(-2, function() {
                            trace("Finished");
                        });
                    });
                });
            });
        });
    });
});
