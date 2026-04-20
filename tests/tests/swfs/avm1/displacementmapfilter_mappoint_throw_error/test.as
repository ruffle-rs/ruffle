function trySetting(f, p) {
    try {
        f.mapPoint = p;
        trace("Set: " + f.mapPoint);
    } catch (e) {
        trace("Caught: " + e + ", " + f.mapPoint);
    }
}

var f = new flash.filters.DisplacementMapFilter();

var fallibleNum = new Object();
fallibleNum.valueOf = function() {
   throw "error";
};

var fallibleNum2 = new Object();
fallibleNum2.valueOf = function() {
   throw "error2";
};

trySetting(f, {x:1, y:2});

var point = new Object();
point.x = fallibleNum;
trySetting(f, point);

var point = new Object();
point.y = fallibleNum;
trySetting(f, point);

point.x = fallibleNum;
point.y = 8;
trySetting(f, point);

point.x = 3;
point.y = 8;
trySetting(f, point);

point.x = 3;
point.y = fallibleNum;
trySetting(f, point);

point.x = fallibleNum;
point.y = fallibleNum2;
trySetting(f, point);

point.x = fallibleNum2;
point.y = fallibleNum;
trySetting(f, point);

point.x = 1;
point.y = 2;
trySetting(f, point);

var point = new Object();
point.x = 1;
trySetting(f, point);

var point = new Object();
point.x = 1;
point.y = 0;
trySetting(f, point);

trySetting(f, true);
trySetting(f, false);
