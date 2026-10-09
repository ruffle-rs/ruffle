// Despite the documentation says that there is no constructor function for the `AsBroadcaster`
// class, Flash accepts expressions like `new AsBroadcaster()`, and a newly-created object is
// returned in such cases.
function AsBroadcaster() {
}

var o = AsBroadcaster;

o.broadcastMessage = ASnative(101, 12);

o.addListener = function(listener) {
    // Dispatch through `removeListener` and `push`, which can be overridden.
    // An already registered listener moves to the end of the list.
    this.removeListener(listener);
    this._listeners.push(listener);
    return true;
};

o.removeListener = function(listener) {
    var listeners = this._listeners;
    // `length` is looked up on every iteration, which is observable via getters.
    for (var i = 0; i < listeners.length; i++) {
        // Listeners are compared with `==`, which may invoke `valueOf`.
        if (listeners[i] == listener) {
            // Dispatch through `splice`, which can be overridden.
            // Only the first matching listener is removed.
            listeners.splice(i, 1);
            return true;
        }
    }
    return false;
};

o.initialize = function(broadcaster) {
    // Assignment order is observable via property enumeration.

    broadcaster.broadcastMessage = ASnative(101, 12);

    // Overriding `AsBroadcaster.addListener` or `AsBroadcaster.removeListener`
    // affects objects initialization.
    broadcaster.addListener = AsBroadcaster.addListener;
    broadcaster.removeListener = AsBroadcaster.removeListener;
    broadcaster._listeners = [];

    // DONT_ENUM, DONT_DELETE, and only visible to SWF 6 and later.
    ASSetPropFlags(broadcaster, "broadcastMessage,addListener,removeListener,_listeners", 131);
};

// DONT_ENUM, DONT_DELETE, and only visible to SWF 6 and later.
ASSetPropFlags(o, null, 131);
