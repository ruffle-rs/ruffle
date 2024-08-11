package avmplus {
    namespace AS3 = "http://adobe.com/AS3/2006/builtin";

    public native function getQualifiedClassName(value:*):String;
    internal native function describeTypeJSON(o:*, flags:uint):Object;

    public const HIDE_NSURI_METHODS:uint    = 0x0001;
    public const INCLUDE_BASES:uint         = 0x0002;
    public const INCLUDE_INTERFACES:uint    = 0x0004;
    public const INCLUDE_VARIABLES:uint     = 0x0008;
    public const INCLUDE_ACCESSORS:uint     = 0x0010;
    public const INCLUDE_METHODS:uint       = 0x0020;
    public const INCLUDE_METADATA:uint      = 0x0040;
    public const INCLUDE_CONSTRUCTOR:uint   = 0x0080;
    public const INCLUDE_TRAITS:uint        = 0x0100;
    public const USE_ITRAITS:uint           = 0x0200;
    public const HIDE_OBJECT:uint           = 0x0400;

    public const FLASH10_FLAGS:uint =   INCLUDE_BASES |
                                        INCLUDE_INTERFACES |
                                        INCLUDE_VARIABLES |
                                        INCLUDE_ACCESSORS |
                                        INCLUDE_METHODS |
                                        INCLUDE_METADATA |
                                        INCLUDE_CONSTRUCTOR |
                                        INCLUDE_TRAITS |
                                        HIDE_NSURI_METHODS |
                                        HIDE_OBJECT;

    internal function copyParams(params: Object, xml: XML) {
        for (var i in params) {
            var param = params[i];
            var elem = <parameter />;
            elem.@index = i + 1;
            elem.@type = param.type;
            elem.@optional = param.optional;
            xml.appendChild(elem);
        }
    }

    internal function copyMetadata(metadata: Array, xml: XML) {
        for each (var md in metadata) {
            var data = <metadata />;
            data.@name = md.name;
            for each (var metaValue in md.value) {
                var elem = <arg />;
                elem.@key = metaValue.key;
                elem.@value = metaValue.value;
                data.appendChild(elem);
            }
            xml.appendChild(data);
        }
    }

    internal function copyUriAndMetadata(data: Object, xml: XML) {
        if (data.uri) {
            xml.@uri = data.uri;
        }
        if (data.metadata) {
            copyMetadata(data.metadata, xml)
        }
    }

    internal function copyTraits(traits: Object, xml: XML) {
        for each (var base in traits.bases) {
            var elem = <extendsClass />;
            elem.@type = base;
            xml.AS3::appendChild(elem);
        }
        for each (var iface in traits.interfaces) {
            var elem = <implementsInterface />;
            elem.@type = iface;
            xml.AS3::appendChild(elem);
        }
        if (traits.constructor) {
            var constructor = <constructor />;
            copyParams(traits.constructor, constructor);
            xml.AS3::appendChild(constructor)
        }

        for each (var variable in traits.variables) {
            var variableXML = (variable.access == "readonly") ? <constant /> : <variable />;
            variableXML.@name = variable.name;
            variableXML.@type = variable.type;
            copyUriAndMetadata(variable, variableXML);
            xml.AS3::appendChild(variableXML);
        }

        for each (var accessor in traits.accessors) {
            var accessorXML = <accessor />;
            accessorXML.@name = accessor.name;
            accessorXML.@access = accessor.access;
            accessorXML.@type = accessor.type;
            accessorXML.@declaredBy = accessor.declaredBy;
            copyUriAndMetadata(accessor, accessorXML);
            xml.AS3::appendChild(accessorXML);
        }

        for each (var method in traits.methods) {
            var methodXML = <method />;
            methodXML.@name = method.name;
            methodXML.@declaredBy = method.declaredBy;
            methodXML.@returnType = method.returnType;

            copyParams(method.parameters, methodXML);
            copyUriAndMetadata(method, methodXML);
            xml.AS3::appendChild(methodXML);
        }

        copyMetadata(traits.metadata, xml);
    }

    public function describeType(value: *, flags: uint):XML {
        var json = describeTypeJSON(value, flags);
        var xml = <type />;
        xml.@name = json.name;
        if (json.traits.bases.length != 0) {
            xml.@base = json.traits.bases[0];
        }
        xml.@isDynamic = json.isDynamic;
        xml.@isFinal = json.isFinal;
        xml.@isStatic = json.isStatic;
        copyTraits(json.traits, xml);

        var jsonITraits = describeTypeJSON(value, flags | USE_ITRAITS);
        if (jsonITraits) {
            var factory = <factory />;
            factory.@type = jsonITraits.name;
            copyTraits(jsonITraits.traits, factory);
            xml.appendChild(factory);
        }

        return xml;
    }
}