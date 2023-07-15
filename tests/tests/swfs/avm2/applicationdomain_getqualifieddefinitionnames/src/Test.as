package {
    public class Test {}
}

import flash.system.ApplicationDomain;
import foo.bar.Baz;

/* This should not show up in the output. */
class NonVisible { }

function check_contains(def_names: Vector.<String>, name: String) : void {
    trace("getQualifiedDefinitionNames() contains " + name + ":", def_names.indexOf(name) != -1);
}

/* Use these to make sure they're included in the SWF. */
Baz;
VisibleFunction;
VisibleNamespace;

var def_names: * = ApplicationDomain.currentDomain.getQualifiedDefinitionNames();

trace("getQualifiedDefinitionNames() is Vector.<String>:", def_names is Vector.<String>);
trace("getQualifiedDefinitionNames().fixed:", def_names.fixed);
trace("getQualifiedDefinitionNames().length:", def_names.length);

/* NOTE: The order of the names should not matter, so we don't test it. */
check_contains(def_names, "Test");
check_contains(def_names, "VisibleFunction");
check_contains(def_names, "VisibleNamespace");
check_contains(def_names, "foo.bar::Baz");
check_contains(def_names, "foo.bar::Internal");

/*
    NOTE: 'getQualifiedDefinitionNames' is able to raise a 'SecurityError',
    however we do not currently test that functionality.
*/
