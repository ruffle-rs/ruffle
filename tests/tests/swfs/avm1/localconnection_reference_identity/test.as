var receiver = new LocalConnection();

receiver.onReceiveData = function(payload) {
    var items = payload.items;

    trace("First marker: " + items[0].marker);
    trace("Second type: " + typeof(items[1]));
    trace("Second marker: " + items[1].marker);

    trace("Self reference: " +
        (payload.cycle.self === payload.cycle));
};

trace("Connected: " + receiver.connect("reference_test"));

var shared = {marker: "shared"};

var cycle = {};
cycle.self = cycle;

var sender = new LocalConnection();

sender.send("reference_test", "onReceiveData", {
    items: [shared, shared],
    cycle: cycle
});
