/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_2_1_5";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Property Accessors";

    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    // go through all Native Function objects, methods, and properties and get their typeof.

    var PROPERTY = new Array();
    var p = 0;

    // try to access properties of primitive types

    PROPERTY[p++] = new Property(  new String("hi"),    "hi",   "hi",   NaN );
    PROPERTY[p++] = new Property(  new Number(NaN),         NaN,    "NaN",    NaN );
    PROPERTY[p++] = new Property(  new Number(3),           3,      "3",    3  );
    PROPERTY[p++] = new Property(  new Boolean(true),        true,      "true",    1 );
    PROPERTY[p++] = new Property(  new Boolean(false),       false,      "false",    0 );

    for ( var i = 0, RESULT; i < PROPERTY.length; i++ ) {
        array[item++] = Assert.expectEq( 
                                        PROPERTY[i].object + ".valueOf()",
                                        PROPERTY[i].value,
                                        (PROPERTY[i].object.valueOf()));

        array[item++] = Assert.expectEq( 
                                        PROPERTY[i].object + ".toString()",
                                        PROPERTY[i].string,
                                        (PROPERTY[i].object.toString()));

    }
    return array;
}


function MyObject( value ) {
    this.value = value;
    this.stringValue = value +"";
    this.numberValue = Number(value);
    return this;
}
function Property( object, value, string, number ) {
    this.object = object;
    this.string = String(value);
    this.number = Number(value);
    this.value = value;
}
