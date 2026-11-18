function inspectObject(label:String, obj:Object):Void {
    trace("[" + label + "] isArray: " + (obj instanceof Array) + " | length: " + obj.length);
    
    // Check if the empty indices are true holes or deserialized explicit properties
    if (obj instanceof Array && obj.length > 2) {
        trace("  Hole Verification via hasOwnProperty:");
        trace("    -> Has explicit index 1 property? " + obj.hasOwnProperty("1"));
        trace("    -> Has explicit index 2 property? " + obj.hasOwnProperty("2"));
    }
    
    // Extract and sort keys to guarantee a deterministic trace output for Ruffle matching
    var keys:Array = new Array();
    for (var k in obj) {
        if (k != "length") {
            keys.push(k);
        }
    }
    keys.sort();
    trace("  Keys found via for..in: " + keys.join(", "));
    
    // Print values associated with active keys
    for (var i = 0; i < keys.length; i++) {
        var keyName = keys[i];
        trace("    -> [" + keyName + "] = " + obj[keyName]);
    }
}
