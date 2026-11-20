this.onUnload = function() {
    trace("onUnload: " + this);
};

function fileName(url) {
    if (typeof url == "string") {
        return url.substr(url.lastIndexOf("/") + 1);
    }
    return url;
}

function describe(label) {
    trace(label + ": " + typeof _level0 + ", " + _level0 + ", " + typeof _level0._url + ", " + fileName(_level0._url) + ", " + _level0._framesloaded);
}

var count = 0;
var interval = setInterval(function() {
    count++;
    describe("after " + count + " intervals");
    if (count == 3) {
        clearInterval(interval);
    }
}, 200);

describe("before");
_level0.childLoaded();
describe("right after");
