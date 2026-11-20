// Exercise both early-return cases in LocalConnection AMF0 Date decoding.
// A Date value is serialized before the global Date binding is replaced.
var OriginalDate = Date;
var sourceDate = new OriginalDate(1234567890000);

var receiver = new LocalConnection();
var sender = new LocalConnection();
var phase = 0;

receiver.onReceiveData = function(received) {
    if (phase == 1) {
        trace("--- LocalConnection with Date = null ---");
        trace("Date type: " + typeof(received.date));
        trace("Date value: " + received.date);

        // Send the next message only after the first one has been decoded,
        // so each message observes a different global Date binding.
        phase = 2;
        _global.Date = {};
        sender.send("amf0_invalid_date_binding", "onReceiveData", { date: sourceDate });
    } else if (phase == 2) {
        trace("--- LocalConnection with Date = {} ---");
        trace("Date type: " + typeof(received.date));
        trace("Date value: " + received.date);
        phase = 3;
    }
};

trace("Connected: " + receiver.connect("amf0_invalid_date_binding"));
_global.Date = null;
phase = 1;
sender.send("amf0_invalid_date_binding", "onReceiveData", { date: sourceDate });
