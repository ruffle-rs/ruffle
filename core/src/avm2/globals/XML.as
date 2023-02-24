package {
    [Ruffle(InstanceAllocator)]
    public final dynamic class XML {
        public function XML(value:* = undefined) {
            this.init(value);
        }

        private native function init(value:*): void;

        AS3 native function name():Object;
        AS3 native function localName():Object;
        AS3 native function toXMLString():String;
        
        AS3 native function toString():String;
    }
}

XML.prototype.name = function():Object {
    var self: XML = this;
    // NOTE - `self.name()` should be sufficient here (and in all of the other methods)
    // However, asc.jar doesn't resolve the 'AS3' namespace when I do
    // 'self.name()' here, which leads to the prototype method invoking
    // itself, instead of the AS3 method.
    return self.AS3::name();
}

XML.prototype.localName = function():Object {
    var self: XML = this;
    return self.AS3::localName();
}

XML.prototype.toXMLString = function():String {
    var self: XML = this;
    return self.AS3::toXMLString();
}
XML.prototype.toString = function():String {
    if (this === XML.prototype) {
        return "";
    }
    var self: XML = this;
    return self.AS3::toString();
}