@PCode {
    Push "The stack was not preserved!"
}
try {
    @PCode {
        Push "The stack was preserved!"
    }
    throw "error";
} catch (e) {
    trace("Caught " + e);
    @PCode {
        PushDuplicate
        Trace
    }
}
trace("Outside catch block:");
@PCode {
    Trace
    Pop
    Pop
}

trace("");
trace("--- and again without throwing ---");
trace("");

@PCode {
    Push "The stack was not preserved!"
}

try {
    @PCode {
        Push "The stack was preserved!"
    }
} catch (e) {
    trace("Caught " + e);
    @PCode {
        PushDuplicate
        Trace
    }
}
trace("Outside catch block:");
@PCode {
    Trace
    Pop
    Pop
}

trace("");
trace("--- and now in reverse ---");
trace("");

@PCode {
    Push 1, 2, 3
}

try {
    @PCode {
        Pop
    }
    throw "error";
} catch (e) {
    trace("Caught " + e);
    @PCode {
        PushDuplicate
        Trace
    }
}
trace("Outside catch block:");
@PCode {
    Trace
}

fscommand("quit");
