function printId3(id3) {
    trace("  id3 = " + id3);
    for (var p in id3) {
        trace("  id3[" + p + "] = " + id3[p]);
    }
}

function testSoundTarget(loadFunc, target, cb) {
    trace("  Testing target " + target);

    if (target !== undefined) {
        sound = new Sound(target);
    } else {
        sound = new Sound();
    }

    trace("    Before loading");
    printId3(sound.id3);

    loadFunc(sound);
}

function testSound(place, cb) {
    trace("// Place " + place);
    var sound = new Sound();
    if (place === 0) sound.id3 = "x";
    sound.onID3 = function(a, b) {
        trace("  onID3 called");
        printId3(sound.id3);
        if (place === 1) sound.id3 = "x";
    };
    sound.onLoad = function(a, b) {
        trace("  loaded");
        printId3(sound.id3);
        if (place === 2) {
            sound.id3 = "x";
            trace("  after set");
            printId3(sound.id3);
        }
        cb();
    }
    sound.loadSound("id3.mp3", false);
}

testSound(0, function() {
    testSound(1, function() {
        testSound(2, function() {
            trace("Finished");
        });
    });
});
