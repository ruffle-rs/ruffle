// An inherited enumerable getter is included by property enumeration, but
// is_property_virtual checks only the receiver's own slot. Retrieving the
// inherited getter throws, so AMF serialization should skip that property
// and continue serializing the object's own data properties.
var getterCalls = 0;

var inheritedProperties = new Object();
inheritedProperties.addProperty("throwingProperty", function() {
    getterCalls++;
    trace("Inherited getter invoked during serialization");
    throw "intentional inherited getter error";
}, null);

var payload = new Object();
payload.__proto__ = inheritedProperties;
payload.visible = "retained";

var receiver = new LocalConnection();
receiver.onReceiveData = function(received) {
    trace("Received visible property: " + received.visible);
    trace("Received throwing property type: " + typeof(received.throwingProperty));
    trace("Getter calls at receive: " + getterCalls);
};
trace("Connected: " + receiver.connect("inherited_throwing_getter"));

var sender = new LocalConnection();
sender.send("inherited_throwing_getter", "onReceiveData", payload);
trace("Getter calls after send: " + getterCalls);
