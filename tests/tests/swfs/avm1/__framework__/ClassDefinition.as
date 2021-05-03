/**
* Defines how a class should be tested.
*/
class ClassDefinition {
    var name : String;
    var constructor : Function;
    var repr : Function;
    var clazz : Object;

    /*
    * Construct a new class definition with the given name of the class,
    * a function that will always return a new constructed object of that class,
    * and a function that can be used to 'represent' the object (such as tracing its properties).
    */
    function ClassDefinition(name, clazz, constructor, repr) {
        this.name = name;
        this.clazz = clazz;
        this.constructor = constructor;
        this.repr = repr;
    }

    private function runStaticOverload(name, argNames, argValues) {
        trace("\n/// Start of static method test " + this.name + "." + name + "(" + argNames + ")\n");

        var constructor = this.constructor;
        var repr = this.repr;
        var clazz = this.clazz;
        var className = this.name;
        var runTest = function(args) {
            var names = "";
            var values = [];

            for (var i = 0; i < args.length; i++) {
                values.push(args[i].value);
                names += args[i].name;

                if (i + 1 < args.length) {
                    names += ", ";
                }
            }

            trace("// " + className + "." + name + "(" + names + ")");

            var result = clazz[name].apply(clazz, values);
            repr("Return value", result);
        };

        Utils.cartesianProduct(argValues, runTest);


        trace("\n/// End of static method test " + this.name + "." + name + "(" + argNames + ")\n");;
    }

    private function runOverload(name, argNames, argValues) {
        trace("\n/// Start of method test " + this.name + "." + name + "(" + argNames + ")\n");

        var constructor = this.constructor;
        var repr = this.repr;
        var className = this.name;
        var runTest = function(args) {
            var names = "";
            var values = [];

            for (var i = 0; i < args.length; i++) {
                values.push(args[i].value);
                names += args[i].name;

                if (i + 1 < args.length) {
                    names += ", ";
                }
            }

            trace("// " + className + "." + name + "(" + names + ")");

            var object = constructor();
            var result = object[name].apply(object, values);
            repr("Return value", result);
            repr("Original object", object);
        };

        Utils.cartesianProduct(argValues, runTest);


        trace("\n/// End of method test " + this.name + "." + name + "(" + argNames + ")\n");
    }

    private function runConstructor(constructor, argNames, argValues) {
        trace("\n/// Start of constructor test new " + this.name + "(" + argNames + ")\n");

        var repr = this.repr;
        var className = this.name;
        var runTest = function(args) {
            var names = "";
            var values = [];

            for (var i = 0; i < args.length; i++) {
                values.push(args[i].value);
                names += args[i].name;

                if (i + 1 < args.length) {
                    names += ", ";
                }
            }

            trace("// new " + className + "(" + names + ")");

            var result = constructor.apply(constructor, values);
            repr("New object", result);
        };

        Utils.cartesianProduct(argValues, runTest);


        trace("\n/// End of constructor test new " + this.name + "(" + argNames + ")\n");
    }

    /**
    * Tests the given static method with the specified signature.
    * 
    * Every valid combination of values for every argument will be iterated upon.
    * 
    * After each test, the return value of the function is printed.
    */
    function testStaticMethod(name, args) {
        var argNames = "";
        var argValues = [];

        this.runStaticOverload(name, argNames, argValues);

        for (var i = 0; i < args.length; i++) {
            var arg = args[i];

            if (i > 0) {
                argNames += ", ";
            }

            if (arg.values.length < 1) {
                throw "Attempted to test static method " + name + " with an empty argument definition!";
            }

            argValues.push(arg.values);
            argNames += arg.name;

            this.runStaticOverload(name, argNames, argValues);
        }
    }

    /**
    * Tests the given method with the specified signature.
    * 
    * Every valid combination of values for every argument will be iterated upon,
    * ran against an always newly constructed object.
    * 
    * After each test, the return value of the method is printed, followed by
    * a repr of the original object.
    */
    function testMethod(name, args) {
        var argNames = "";
        var argValues = [];

        this.runOverload(name, argNames, argValues);

        for (var i = 0; i < args.length; i++) {
            var arg = args[i];

            if (i > 0) {
                argNames += ", ";
            }

            if (arg.values.length < 1) {
                throw "Attempted to test function " + name + " with an empty argument definition!";
            }

            argValues.push(arg.values);
            argNames += arg.name;

            this.runOverload(name, argNames, argValues);
        }
    }

    /**
    * Tests the given constructor with the specified signature.
    * 
    * Every valid combination of values for every argument will be iterated upon,
    * and provided to the given constructor.
    *
    * Due to AVM1 limitations, the constructor must take in args, switch on the length,
    * and call the actual class constructor based on the length.
    * See the Date test for an example of this.
    * 
    * After each test, the return value of the constructor is printed using repr.
    */
    function testConstructor(constructor, args) {
        var argNames = "";
        var argValues = [];

        this.runConstructor(constructor, argNames, argValues);

        for (var i = 0; i < args.length; i++) {
            var arg = args[i];

            if (i > 0) {
                argNames += ", ";
            }

            if (arg.values.length < 1) {
                throw "Attempted to test constructor with an empty argument definition!";
            }

            argValues.push(arg.values);
            argNames += arg.name;

            this.runConstructor(constructor, argNames, argValues);
        }
    }
}