package {
    [Ruffle(InstanceAllocator)]
    [Ruffle(CallHandler)]
    public final dynamic class XML {
        AS3 static function setSettings(settings:Object = null): void {
            if (settings == null) {
                settings = XML.AS3::defaultSettings();
            }
            if ("ignoreComments" in settings) {
                XML.ignoreComments = settings.ignoreComments;
            }
            if ("ignoreProcessingInstructions" in settings) {
                XML.ignoreProcessingInstructions = settings.ignoreProcessingInstructions;
            }
            if ("ignoreWhitespace" in settings) {
                XML.ignoreWhitespace = settings.ignoreWhitespace;
            }
            if ("prettyIndent" in settings) {
                XML.prettyIndent = settings.prettyIndent;
            }
            if ("prettyPrinting" in settings) {
                XML.prettyPrinting = settings.prettyPrinting;
            }
        }

        AS3 static function settings():Object {
            return {
                ignoreComments: XML.ignoreComments,
                ignoreProcessingInstructions: XML.ignoreProcessingInstructions,
                ignoreWhitespace: XML.ignoreWhitespace,
                prettyIndent: XML.prettyIndent,
                prettyPrinting: XML.prettyPrinting
            };
        }

        AS3 static function defaultSettings():Object {
            return {
                ignoreComments: true,
                ignoreProcessingInstructions: true,
                ignoreWhitespace: true,
                prettyIndent: 2,
                prettyPrinting: true
            };
        }

        public function XML(value:* = void 0) {
            this.init(value, XML.ignoreComments, XML.ignoreProcessingInstructions, XML.ignoreWhitespace);
        }

        private native function init(value:*, ignoreComments:Boolean, ignoreProcessingInstructions:Boolean, ignoreWhitespace:Boolean):void;

        // XML functions

        AS3 native function hasComplexContent():Boolean;
        AS3 native function hasSimpleContent():Boolean;

        AS3 native function attribute(name:*):XMLList;
        AS3 native function child(name:*):XMLList;
        AS3 native function children():XMLList;
        AS3 native function contains(value:*):Boolean;
        AS3 native function copy():XML;
        AS3 native function descendants(name:Object = "*"):XMLList;
        AS3 native function length():int;
        AS3 native function normalize(): XML;
        AS3 native function parent():*;
        AS3 native function text():XMLList;
        AS3 native function toString():String;
        AS3 native function toXMLString():String;

        AS3 native function attributes():XMLList;
        AS3 native function comments():XMLList;
        AS3 native function elements(name:* = "*"):XMLList;
        AS3 native function processingInstructions(name:* = "*"):XMLList;

        AS3 function valueOf():XML {
            return this;
        }

        AS3 function toJSON(k:String) : * {
            return this.toJSON(k);
        }

        // undocumented functions
        AS3 native function notification():Function;
        AS3 native function setNotification(f:Function):*;

        // XML settings functions

        public static native function get ignoreComments():Boolean;
        public static native function set ignoreComments(value:Boolean):void;

        public static native function get ignoreProcessingInstructions():Boolean;
        public static native function set ignoreProcessingInstructions(value:Boolean):void;

        public static native function get ignoreWhitespace():Boolean;
        public static native function set ignoreWhitespace(value:Boolean):void;

        public static native function get prettyPrinting():Boolean;
        public static native function set prettyPrinting(value:Boolean):void;

        public static native function get prettyIndent():int;
        public static native function set prettyIndent(value:int):void;

        // These are also XML functions, but are all marked NativeCallable so that
        // XMLList code can call them. See xml_list.rs for a more detailed explanation.

        private native function namespace_internal_impl(hasPrefix:Boolean, prefix:String = null):*;

        [Ruffle(NativeCallable)]
        AS3 function namespace(prefix:* = null):* {
            return namespace_internal_impl(arguments.length > 0, prefix);
        }

        [Ruffle(NativeCallable)]
        AS3 native function addNamespace(ns:*):XML;

        [Ruffle(NativeCallable)]
        AS3 native function appendChild(child:*):XML;

        [Ruffle(NativeCallable)]
        AS3 native function childIndex():int;

        [Ruffle(NativeCallable)]
        AS3 native function inScopeNamespaces():Array;

        [Ruffle(NativeCallable)]
        AS3 native function insertChildAfter(child1:*, child2:*):*;

        [Ruffle(NativeCallable)]
        AS3 native function insertChildBefore(child1:*, child2:*):*;

        [Ruffle(NativeCallable)]
        AS3 native function localName():Object;

        [Ruffle(NativeCallable)]
        AS3 native function name():Object;

        [Ruffle(NativeCallable)]
        AS3 native function namespaceDeclarations():Array;

        [Ruffle(NativeCallable)]
        AS3 native function nodeKind():String;

        [Ruffle(NativeCallable)]
        AS3 native function prependChild(child:*):XML;

        [Ruffle(NativeCallable)]
        AS3 native function removeNamespace(ns:*):XML;

        // NOTE: Docs lie, value can be anything not just XML.
        [Ruffle(NativeCallable)]
        AS3 native function replace(propertyName:*, value:*):XML;

        [Ruffle(NativeCallable)]
        AS3 native function setChildren(value:*):XML;

        [Ruffle(NativeCallable)]
        AS3 native function setLocalName(name:*):void;

        [Ruffle(NativeCallable)]
        AS3 native function setName(name:*):void;

        [Ruffle(NativeCallable)]
        AS3 native function setNamespace(ns:*):void;

        // Prototype methods

        prototype.hasComplexContent = function():Boolean {
            var self:XML = this;
            return self.AS3::hasComplexContent();
        }

        prototype.hasSimpleContent = function():Boolean {
            var self:XML = this;
            return self.AS3::hasSimpleContent();
        }

        prototype.name = function():Object {
            var self:XML = this;
            // NOTE - `self.name()` should be sufficient here (and in all of the other methods)
            // However, asc.jar doesn't resolve the 'AS3' namespace when I do
            // 'self.name()' here, which leads to the prototype method invoking
            // itself, instead of the AS3 method.
            return self.AS3::name();
        };

        prototype.setName = function(name:*):void {
            var self:XML = this;
            self.AS3::setName(name);
        };

        prototype.namespace = function(prefix:* = null):* {
            var self:XML = this;
            return self.AS3::namespace.apply(self, arguments);
        };

        prototype.addNamespace = function(ns:*):XML {
            var self:XML = this;
            return self.AS3::addNamespace(ns);
        };

        prototype.setNamespace = function(ns:*):void {
            var self:XML = this;
            self.AS3::setNamespace(ns);
        };

        prototype.removeNamespace = function(ns:*):XML {
            var self:XML = this;
            return self.AS3::removeNamespace(ns);
        };

        prototype.namespaceDeclarations = function():Array {
            var self:XML = this;
            return self.AS3::namespaceDeclarations();
        };

        prototype.inScopeNamespaces = function():Array {
            var self:XML = this;
            return self.AS3::inScopeNamespaces();
        };

        prototype.localName = function():Object {
            var self:XML = this;
            return self.AS3::localName();
        };

        prototype.toXMLString = function():String {
            var self:XML = this;
            return self.AS3::toXMLString();
        };

        prototype.child = function(name:*):XMLList {
            var self:XML = this;
            return self.AS3::child(name);
        };

        prototype.childIndex = function():int {
            var self:XML = this;
            return self.AS3::childIndex();
        };

        prototype.children = function():XMLList {
            var self:XML = this;
            return self.AS3::children();
        };

        prototype.contains = function(value:*):Boolean {
            var self:XML = this;
            return self.AS3::contains(value);
        };

        prototype.copy = function():XML {
            var self:XML = this;
            return self.AS3::copy();
        }

        prototype.parent = function():* {
            var self:XML = this;
            return self.AS3::parent();
        };

        prototype.elements = function(name:* = "*"):XMLList {
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

        prototype.appendChild = function(child:*):XML {
            var self:XML = this;
            return self.AS3::appendChild(child);
        };

        prototype.prependChild = function(child:*):XML {
            var self:XML = this;
            return self.AS3::prependChild(child);
        };

        prototype.descendants = function(name:* = "*"):XMLList {
            var self:XML = this;
            return self.AS3::descendants(name);
        };

        prototype.text = function():XMLList {
            var self:XML = this;
            return self.AS3::text();
        };

        prototype.normalize = function():XML {
            var self:XML = this;
            return self.AS3::normalize();
        };

        prototype.length = function():int {
            var self:XML = this;
            return self.AS3::length();
        }

        prototype.toJSON = function(k:String):* {
            return "XML";
        };

        prototype.comments = function():XMLList {
            var self:XML = this;
            return self.AS3::comments();
        }

        prototype.processingInstructions = function(name:* = "*"):XMLList {
            var self:XML = this;
            return self.AS3::processingInstructions(name);
        }

        prototype.insertChildAfter = function(child1:*, child2:*):* {
            var self:XML = this;
            return self.AS3::insertChildAfter(child1, child2);
        }

        prototype.insertChildBefore = function(child1:*, child2:*):* {
            var self:XML = this;
            return self.AS3::insertChildBefore(child1, child2);
        }

        prototype.replace = function(propertyName:*, value:*):XML {
            var self:XML = this;
            return self.AS3::replace(propertyName, value);
        }

        prototype.setChildren = function(value:*):XML {
            var self:XML = this;
            return self.AS3::setChildren(value);
        }

        prototype.setLocalName = function(name:*):void {
            var self:XML = this;
            self.AS3::setLocalName(name);
        }

        prototype.setPropertyIsEnumerable("hasComplexContent", false);
        prototype.setPropertyIsEnumerable("hasSimpleContent", false);
        prototype.setPropertyIsEnumerable("name", false);
        prototype.setPropertyIsEnumerable("setName", false);
        prototype.setPropertyIsEnumerable("namespace", false);
        prototype.setPropertyIsEnumerable("addNamespace", false);
        prototype.setPropertyIsEnumerable("setNamespace", false);
        prototype.setPropertyIsEnumerable("removeNamespace", false);
        prototype.setPropertyIsEnumerable("namespaceDeclarations", false);
        prototype.setPropertyIsEnumerable("inScopeNamespaces", false);
        prototype.setPropertyIsEnumerable("localName", false);
        prototype.setPropertyIsEnumerable("toXMLString", false);
        prototype.setPropertyIsEnumerable("child", false);
        prototype.setPropertyIsEnumerable("childIndex", false);
        prototype.setPropertyIsEnumerable("children", false);
        prototype.setPropertyIsEnumerable("contains", false);
        prototype.setPropertyIsEnumerable("copy", false);
        prototype.setPropertyIsEnumerable("parent", false);
        prototype.setPropertyIsEnumerable("elements", false);
        prototype.setPropertyIsEnumerable("toString", false);
        prototype.setPropertyIsEnumerable("attributes", false);
        prototype.setPropertyIsEnumerable("attribute", false);
        prototype.setPropertyIsEnumerable("nodeKind", false);
        prototype.setPropertyIsEnumerable("appendChild", false);
        prototype.setPropertyIsEnumerable("prependChild", false);
        prototype.setPropertyIsEnumerable("descendants", false);
        prototype.setPropertyIsEnumerable("text", false);
        prototype.setPropertyIsEnumerable("normalize", false);
        prototype.setPropertyIsEnumerable("length", false);
        prototype.setPropertyIsEnumerable("toJSON", false);
        prototype.setPropertyIsEnumerable("comments", false);
        prototype.setPropertyIsEnumerable("processingInstructions", false);
        prototype.setPropertyIsEnumerable("insertChildAfter", false);
        prototype.setPropertyIsEnumerable("insertChildBefore", false);
        prototype.setPropertyIsEnumerable("replace", false);
        prototype.setPropertyIsEnumerable("setChildren", false);
        prototype.setPropertyIsEnumerable("setLocalName", false);

        XML.settings = function() {
            return XML.AS3::settings();
        }

        XML.setSettings = function(v:* = void 0) {
            XML.AS3::setSettings(v)
        }

        XML.defaultSettings = function() {
            return XML.AS3::defaultSettings();
        }

        public static const length:* = 1;
    }
}
