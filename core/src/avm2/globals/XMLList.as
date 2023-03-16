package {
    [Ruffle(InstanceAllocator)]
    public final dynamic class XMLList {

        public function XMLList(value:* = undefined) {
            this.init(value);
        }

        private native function init(value:*): void;

        AS3 native function hasSimpleContent():Boolean;
        AS3 native function length():int;
        AS3 native function children():XMLList;
        AS3 native function attribute(name:*):XMLList;
        AS3 native function attributes():XMLList;
        AS3 native function descendants(name:Object = "*"):XMLList;
        AS3 native function toXMLString():String;

        AS3 native function toString():String;

        // The following native methods are not declared in the documentation,
        // but still exist
        AS3 native function name(): Object;

        prototype.hasSimpleContent = function():Boolean {
            var self:XMLList = this;
            // NOTE - `self.hasSimpleContent()` should be sufficient here (and in all of the other methods)
            // However, asc.jar doesn't resolve the 'AS3' namespace when I do
            // 'self.hasSimpleContent()' here, which leads to the prototype method invoking
            // itself, instead of the AS3 method.
            return self.AS3::hasSimpleContent();
        }

        prototype.length = function():int {
            var self:XMLList = this;
            return self.AS3::length();
        }

        prototype.children = function():XMLList {
            var self:XMLList = this;
            return self.AS3::children();
        }

        prototype.attribute = function(name:*):XMLList {
            var self:XMLList = this;
            return self.AS3::attribute(name);
        }

        prototype.attributes = function():XMLList {
            var self:XMLList = this;
            return self.AS3::attributes();
        }

        prototype.toString = function():String {
            var self:XMLList = this;
            return self.AS3::toString();
        }

        prototype.toXMLString = function():String {
            var self:XMLList = this;
            return self.AS3::toXMLString();
        }

        prototype.name = function(): Object {
            var self:XMLList = this;
            return self.AS3::name();
        }

        prototype.descendants = function(name:Object):XMLList {
            var self:XMLList = this;
            return self.AS3::descendants(name);
        }
    }
}