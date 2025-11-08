var doPrint = function(string) {
    if(string.indexOf("instance") != -1) {
        trace(string.substring(0,string.indexOf("instance")) + "instanceXX");
    } else {
        trace(string);
    }
};
for(var key in _root) {
    if(key != "$version") {
        doPrint("prop: " + key);
        doPrint("object: " + _root[key]);
        doPrint("object name: " + _root[key]._name);
    }
}
