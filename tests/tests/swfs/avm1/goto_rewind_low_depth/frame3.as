if (!_global.done) {
    trace("== frame 3"); rep(this, "f3"); mark(this, 3);
    r3.gotoAndStop(3); repr(r3, "f3 r3 frame 3");
    r3.gotoAndStop(2); repr(r3, "f3 r3 frame 3 to 2");
    r4.gotoAndStop(4); repr(r4, "f3 r4 frame 4");
    r4.gotoAndStop(2); repr(r4, "f3 r4 frame 4 to 2");
    for (var i = 0; i < K.length; i++) { this["k" + K[i]].gotoAndStop(1); }
    _global.done = true;
    gotoAndStop(1);
    trace("== after gotoAndStop(1)"); rep(this, "g");
    var self = this; var n = 0;
    this.onEnterFrame = function() {
        n++;
        trace("== enterFrame " + n); rep(self, "e" + n);
        if (n >= 2) { delete self.onEnterFrame; trace("== done"); }
    };
}
