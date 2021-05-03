/**
* Defines how an argument to a function should be tested.
*/
class ArgumentDefinition {
    var name : String;
    var values : Array;

    /*
    * Construct a new argument definition with the given name of the argument.
    *
    * If you include an 'inherit' argument, every value from that will be copied to this.
    */
    function ArgumentDefinition(name, inherit) {
        this.name = name;
        this.values = [];

        if (typeof inherit === "object") {
            for (var i = 0; i < inherit.values.length; i++) {
                this.values.push(inherit.values[i]);
            }
        }
    }

    /**
    * Adds the specified value as a potential value that may be used for this argument.
    *
    * If the value cannot be expressed via .toString(), then please specify a name manually.
    * For example, {} would print [object Object] and so you should use "{}" as a name.
    *
    * Returns this object, for chaining method calls.
    */
    function add(value, name) {
        if (name === undefined) {
            name = Utils.stringify(value);
        }
        this.values.push({name: name, value: value});
        return this;
    }
}