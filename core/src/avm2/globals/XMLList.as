package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public final dynamic class XMLList {

        public function XMLList(value:* = void 0) {
            this.init(value, XML.ignoreComments, XML.ignoreProcessingInstructions, XML.ignoreWhitespace);
        }

        private native function init(value:*, ignoreComments:Boolean, ignoreProcessingInstructions:Boolean, ignoreWhitespace:Boolean): void;

        AS3 native function hasComplexContent():Boolean;
        AS3 native function hasSimpleContent():Boolean;
        AS3 native function length():int;
        AS3 native function child(name:Object):XMLList;
        AS3 native function children():XMLList;
        AS3 native function contains(value:*):Boolean;
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
        AS3 native function elements(name:* = "*"):XMLList;
        AS3 native function normalize():XMLList;

        // The following native methods are not declared in the documentation,
        // but still exist
        AS3 native function addNamespace(ns:*):XML;
        AS3 native function appendChild(child:*):XML;
        AS3 native function childIndex():int;
        AS3 native function inScopeNamespaces():Array;
        AS3 native function insertChildAfter(child1:*, child2:*):*;
        AS3 native function insertChildBefore(child1:*, child2:*):*;
        AS3 native function localName():Object
        AS3 native function name(): Object;
        private native function namespace_internal_impl(hasPrefix:Boolean, prefix:String = null):*;
        AS3 function namespace(prefix:* = null):* {
            return namespace_internal_impl(arguments.length > 0, prefix);
        }
        AS3 native function namespaceDeclarations():Array;
        AS3 native function nodeKind(): String;
        AS3 native function prependChild(child:*):XML;
        AS3 native function removeNamespace(ns:*):XML;
        AS3 native function replace(propertyName:*, value:*):XML;
        AS3 native function setChildren(value:*):XML;
        AS3 native function setLocalName(name:*):void;
        AS3 native function setName(name:*):void;
        AS3 native function setNamespace(ns:*):void;

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

        prototype.contains = function(value:*):Boolean {
            var self:XMLList = this;
            return self.AS3::contains(value);
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

        prototype.addNamespace = function(ns:*):XML {
            var self:XMLList = this;
            return self.AS3::addNamespace(ns);
        }

        prototype.appendChild = function(child:*):XML {
            var self:XMLList = this;
            return self.AS3::appendChild(child);
        }

        prototype.childIndex = function():int {
            var self:XMLList = this;
            return self.AS3::childIndex();
        }

        prototype.inScopeNamespaces = function():Array {
            var self:XMLList = this;
            return self.AS3::inScopeNamespaces();
        }

        prototype.insertChildAfter = function(child1:*, child2:*):* {
            var self:XMLList = this;
            return self.AS3::insertChildAfter(child1, child2);
        }

        prototype.insertChildBefore = function(child1:*, child2:*):* {
            var self:XMLList = this;
            return self.AS3::insertChildBefore(child1, child2);
        }

        prototype.localName = function():Object {
            var self:XMLList = this;
            return self.AS3::localName();
        }

        prototype.name = function(): Object {
            var self:XMLList = this;
            return self.AS3::name();
        }

        prototype.namespace = function(prefix:* = null):* {
            var self:XMLList = this;
            return self.AS3::namespace.apply(self, arguments);
        }

        prototype.namespaceDeclarations = function():Array {
            var self:XMLList = this;
            return self.AS3::namespaceDeclarations();
        }

        prototype.nodeKind = function():String {
            var self:XMLList = this;
            return self.AS3::nodeKind();
        }

        prototype.prependChild = function(child:*):XML {
            var self:XMLList = this;
            return self.AS3::prependChild(child);
        }

        prototype.removeNamespace = function(ns:*):XML {
            var self:XMLList = this;
            return self.AS3::removeNamespace(ns);
        }

        prototype.replace = function(propertyName:*, value:*):XML {
            var self:XMLList = this;
            return self.AS3::replace(propertyName, value);
        }

        prototype.setChildren = function(value:*):XML {
            var self:XMLList = this;
            return self.AS3::setChildren(value);
        }

        prototype.setLocalName = function(name:*):void {
            var self:XMLList = this;
            self.AS3::setLocalName(name);
        }

        prototype.setName = function(name:*):void {
            var self:XMLList = this;
            self.AS3::setName(name);
        }

        prototype.setNamespace = function(ns:*):void {
            var self:XMLList = this;
            self.AS3::setNamespace(ns);
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

        prototype.elements = function(name:* = "*"):XMLList {
            var self:XMLList = this;
            return self.AS3::elements(name);
        }

        prototype.normalize = function():XMLList {
            var self:XMLList = this;
            return self.AS3::normalize();
        }

        public static const length:* = 1;
    }
}
