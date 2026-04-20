function printId3(id3) {
    trace("      id3 = " + id3);
    trace("      typeof id3 = " + (typeof id3));
    trace("      id3 == undefined = " + (id3 == undefined));
    trace("      id3 == null = " + (id3 == null));
    trace("      id3 == 0 = " + (id3 == 0));
    trace("      id3 === undefined = " + (id3 === undefined));
    trace("      id3 === null = " + (id3 === null));
    trace("      id3 === 0 = " + (id3 === 0));
    for (var p in id3) {
        trace("      id3[" + p + "] = " + id3[p]);
        trace("      typeof id3[" + p + "] = " + (typeof id3[p]));
        if (typeof id3[p] == "object") {
            trace("      id3[" + p + "].length = " + id3[p].length);
            for (var p2 in id3[p]) {
                trace("      id3[" + p + "][" + p2 + "] = " + id3[p][p2]);
                trace("      typeof id3[" + p + "][" + p2 + "] = " + (typeof id3[p][p2]));
            }
        }
    }
}

function testSoundTarget(loadFunc, target, cb) {
    trace("  Testing target " + target);
    var sound;

    if (target !== undefined) {
        sound = new Sound(target);
    } else {
        sound = new Sound();
    }

    sound.onID3 = function(a, b) {
        trace("      onID3 called: " + a + ", " + b);
        printId3(sound.id3);
    };
    sound.onLoad = function(a, b) {
        trace("    After loading: " + a + ", " + b);
        printId3(sound.id3);
        cb();
    }

    trace("    Before loading");
    printId3(sound.id3);

    loadFunc(sound);
}

function testSound(loadFunc, cb) {
    testSoundTarget(loadFunc, undefined, function() {
        testSoundTarget(loadFunc, _root, cb);
    });
}

trace("Load id3, true");
testSound(function(s) { s.loadSound("id3.mp3", true); }, function() {
    trace("Load id3, false");
    testSound(function(s) { s.loadSound("id3.mp3", false); }, function() {
        trace("Load noid3, true");
        testSound(function(s) { s.loadSound("noid3.mp3", true); }, function() {
            trace("Load noid3, false");
            testSound(function(s) { s.loadSound("noid3.mp3", false); }, function() {
                trace("Finished");
            });
        });
    });
});
