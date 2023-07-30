/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "10.1.6";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Activation Object";


    //    The current object has an arguments property.
    var arguments = "FAILED!";

    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;


    var ARG_STRING = "value of the argument property";

    array[item++] = Assert.expectEq( 
                                    "(new TestObject(0,1,2,3,4,5)).length",
                                    6,
                                    (new TestObject(0,1,2,3,4,5)).length );

    for ( i = 0; i < 6; i++ ) {

        array[item++] = Assert.expectEq( 
                                        "(new TestObject(0,1,2,3,4,5))["+i+"]",
                                        i,
                                        (new TestObject(0,1,2,3,4,5))[i]);
    }


    //    The current object already has an arguments property.

    array[item++] = Assert.expectEq( 
                                    "(new AnotherTestObject(1,2,3)).arguments",
                                    ARG_STRING,
                                    (new AnotherTestObject(1,2,3)).arguments );

    //  The function invoked with [[Call]]

    array[item++] = Assert.expectEq( 
                                    "TestFunction(1,2,3)",
                                    ARG_STRING,
                                    TestFunction() + '' );


    function TestObject() {
        // cn:  __proto__ not ecma3 compliant
        //this.__proto__ = new Prototype();
        this.constructor.prototype.arguments = ARG_STRING;
        return arguments;
    }
    function AnotherTestObject() {
        // cn:  __proto__ not ecma3 compliant
        //this.__proto__ = new Prototype();
        this.constructor.prototype.arguments = ARG_STRING;
        return this;
    }
    function TestFunction() {
        arguments[0] = ARG_STRING;
        return arguments;
    }
    function AnotherTestFunction() {
        // cn:  __proto__ not ecma3 compliant
        //this.__proto__ = new Prototype();
        this.constructor.prototype.arguments = ARG_STRING;
        return this;
    }

    return ( array );
}
