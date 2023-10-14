/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "10.1.8";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Arguments Object";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    var ARG_STRING = "value of the argument property";

    array[item++] = Assert.expectEq( 
                                    "GetCallee()",
                                    GetCallee,
                                    GetCallee() );


    var LENGTH = GetLength ( 0,1,2,3,4 );
    array[item++] = Assert.expectEq( 
                                    "GetLength(0,1,2,3,4)",
                                    5,
                                    LENGTH );

    var ARGUMENTS = GetArguments( 0,1,2,3,4,5 );
    for ( var i = 0; i < 5; i++ ) {
        array[item++] = Assert.expectEq( 
                                        "GetArguments()",
                                        i,
                                        ARGUMENTS[i] );
    }

    function TestFunction() {
        var arg_proto = arguments.constructor.prototype;
    }
    function GetCallee() {
        var c = arguments.callee;
        return c;
    }
    function GetArguments() {
        var a = arguments;
        return a;
    }
    function GetLength() {
        var l = arguments.length;
        return l;
    }

    function AnotherTestFunction() {
        this.constructor.prototype.arg_proto = arguments.constructor.prototype;
        return this;
    }

    return ( array );
}
