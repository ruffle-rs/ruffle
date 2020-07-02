class LoggingWatcher {
    var count = 0;
    var value = true;

    function LoggingWatcher() {
        trace("// this.watch(\"value\", this.log)");
        this.watch("value", this.log);
    }

    function log(property, oldValue, newValue, bounds) {
        var userdata;
        if (typeof bounds === "object") {
            userdata = "{ ";
            for (var key in bounds) {
                userdata += key + "=" + bounds[key] + " ";
            }
            userdata += "}";
        } else {
            userdata = bounds;
        }
        this.count++;
        trace("LoggingWatcher count " + this.count + ": " + property + " changed from " + oldValue + " to " + newValue + " with userdata " + userdata);
        return newValue;
    }
}