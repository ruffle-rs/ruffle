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

XML.prototype.name = function():Object { return this.AS3::name(); }
XML.prototype.localName = function():Object {return this.AS3::localName(); }
XML.prototype.toXMLString = function():String { return this.AS3::toXMLString(); }
XML.prototype.toString = function():String {
    if (this === XML.prototype) {
        return "";
    }
    return this.AS3::toString();
}