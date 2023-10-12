package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public final dynamic class XMLList {

        public function XMLList(value:* = undefined) {
            this.init(value, XML.ignoreComments, XML.ignoreProcessingInstructions, XML.ignoreWhitespace);
        }

        private native function init(value:*, ignoreComments:Boolean, ignoreProcessingInstructions:Boolean, ignoreWhitespace:Boolean): void;

        AS3 native function hasComplexContent():Boolean;
        AS3 native function hasSimpleContent():Boolean;
        AS3 native function length():int;
        AS3 native function child(name:Object):XMLList;
        AS3 native function children():XMLList;
        AS3 native function copy():XMLList;
        AS3 native function attribute(name:*):XMLList;
        AS3 native function attributes():XMLList;
        AS3 native function descendants(name:* = "*"):XMLList;
        AS3 native function text():XMLList;
        AS3 native function toXMLString():String;
        AS3 native function toString():String;
        AS3 native function comments():XMLList;
        AS3 native function parent():*;
        AS3 native function processingInstructions(name:* = "*"):XMLList;

        // The following native methods are not declared in the documentation,
        // but still exist
        AS3 native function name(): Object;

        AS3 function toJSON(k:String) : * {
            return this.toJSON(k);
        }

        AS3 function valueOf():XMLList {
            return this;
        }

        prototype.hasComplexContent = function():Boolean {
            var self:XMLList = this;
            return self.AS3::hasComplexContent();
        }

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

        prototype.child = function(name:Object):XMLList {
            var self:XMLList = this;
            return self.AS3::child(name);
        };

        prototype.children = function():XMLList {
            var self:XMLList = this;
            return self.AS3::children();
        }

        prototype.copy = function():XMLList {
            var self:XMLList = this;
            return self.AS3::copy();
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

        prototype.descendants = function(name:* = "*"):XMLList {
            var self:XMLList = this;
            return self.AS3::descendants(name);
        }

        prototype.text = function():XMLList {
            var self:XMLList = this;
            return self.AS3::text();
        }

        prototype.comments = function():XMLList {
            var self:XMLList = this;
            return self.AS3::comments();
        }

        prototype.parent = function():* {
            var self:XMLList = this;
            return self.AS3::parent();
        }

        prototype.toJSON = function(k:String):* {
            return "XMLList";
        };

        prototype.processingInstructions = function(name:* = "*"):XMLList {
            var self:XMLList = this;
            return self.AS3::processingInstructions(name);
        }

        public static const length:int = 1;
    }
}
