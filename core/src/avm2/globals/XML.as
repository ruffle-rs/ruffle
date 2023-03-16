package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public final dynamic class XML {
        public function XML(value:* = undefined) {
            this.init(value);
        }

        private native function init(value:*):void;

        AS3 native function name():Object;
        AS3 native function namespace(prefix:String = null):*;
        AS3 native function localName():Object;
        AS3 native function toXMLString():String;
        AS3 native function child(name:Object):XMLList;
        AS3 native function children():XMLList;
        AS3 native function elements(name:*):XMLList;
        AS3 native function attributes():XMLList;
        AS3 native function attribute(name:*):XMLList;
        AS3 native function nodeKind():String;
        AS3 native function appendChild(child:Object):XML;
        AS3 native function descendants(name:Object = "*"):XMLList;


        AS3 native function toString():String;

        prototype.name = function():Object {
            var self:XML = this;
            // NOTE - `self.name()` should be sufficient here (and in all of the other methods)
            // However, asc.jar doesn't resolve the 'AS3' namespace when I do
            // 'self.name()' here, which leads to the prototype method invoking
            // itself, instead of the AS3 method.
            return self.AS3::name();
        };

        prototype.namespace = function(prefix:String = null):* {
            var self:XML = this;
            return self.AS3::namespace(prefix);
        }

        prototype.localName = function():Object {
            var self:XML = this;
            return self.AS3::localName();
        };

        prototype.toXMLString = function():String {
            var self:XML = this;
            return self.AS3::toXMLString();
        };

        prototype.child = function(name:Object):XMLList {
            var self:XML = this;
            return self.AS3::child(name);
        };

        prototype.children = function():XMLList {
            var self:XML = this;
            return self.AS3::children();
        };

        prototype.elements = function(name:*):XMLList {
            var self:XML = this;
            return self.AS3::elements(name);
        }

        prototype.toString = function():String {
            if (this === prototype) {
                return "";
            }
            var self:XML = this;
            return self.AS3::toString();
        };

        prototype.attributes = function():XMLList {
            var self:XML = this;
            return self.AS3::attributes();
        };

        prototype.attribute = function(name:*):XMLList {
            var self:XML = this;
            return self.AS3::attribute(name);
        };

        prototype.nodeKind = function():String {
            var self:XML = this;
            return self.AS3::nodeKind();
        };
        
        prototype.appendChild = function(child:Object):XML {
            var self:XML = this;
            return self.AS3::appendChild(child);
        };

        prototype.descendants = function(name:Object):XMLList {
            var self:XML = this;
            return self.AS3::descendants(name);
        };
    }
}
