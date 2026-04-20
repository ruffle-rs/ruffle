function visibleForFlags(flags) {
    var o = {};
    o.test = 42;
    ASSetPropFlags(o, "test", flags);
    return o.test === 42;
}

function detectSwfVersion() {
    if (visibleForFlags(16384)) {
        return 10;
    }
    if (visibleForFlags(8192)) {
        return 9;
    }
    if (visibleForFlags(4096)) {
        return 8;
    }
    if (visibleForFlags(1280)) {
        return 7;
    }
    if (visibleForFlags(128)) {
        return 6;
    }
    return 5;
}

trace("Child: " + detectSwfVersion());
