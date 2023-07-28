/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
 // TODO: REVIEW AS4 CONVERSION ISSUE 
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "number-003";
//     var VERSION = "ECMA_4";
//     var TITLE   = "Exceptions for Number.valueOf()";

    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;


    var result = "Failed";
    var exception = "No exception thrown";
    var expect = "Passed";

    try {
        VALUE_OF = Number.prototype.valueOf;
        OBJECT = new String("Infinity");
        OBJECT.valueOf = VALUE_OF;
        result = OBJECT.valueOf();
    } catch ( e ) {
        result = expect;
        exception = e.toString();
    }

    array[item++] = Assert.expectEq(
        // SECTION,
        "Assigning Number.prototype.valueOf as the valueOf of a String object " +
        " (threw " + Utils.referenceError(exception) +")",
        expect,
        result );

    // new Number()
    try {
        VALUE_OF = Number.prototype.valueOf;
        OBJECT = new Number();
        OBJECT.valueOf = VALUE_OF;
        result = OBJECT.valueOf();
    } catch ( e1:ReferenceError ) {
        result = expect;
        exception = e1.toString();
    }

    array[item++] = Assert.expectEq(
       // SECTION,
        "Assigning Number.prototype.valueOf as the valueOf of new Number() " +
        " (threw " + Utils.referenceError(exception) +")",
        expect,
        result );

    // new Number(4)
    try {
        VALUE_OF = Number.prototype.valueOf;
        OBJECT = new Number(4);
        OBJECT.valueOf = VALUE_OF;
        result = OBJECT.valueOf();
    } catch ( e2 ) {
        result = expect;
        exception = e2.toString();
    }

    array[item++] = Assert.expectEq(
       // SECTION,
        "Assigning Number.prototype.valueOf as the valueOf of new Number(4) " +
        " (threw " + Utils.referenceError(exception) +")",
        expect,
        result );

    // 4
    try {
        VALUE_OF = Number.prototype.valueOf;
        OBJECT = 4;
        OBJECT.valueOf = VALUE_OF;
        result = OBJECT.valueOf();
    } catch ( e3 ) {
        result = expect;
        exception = e3.toString();
    }

    array[item++] = Assert.expectEq(
       // SECTION,
        "Assigning Number.prototype.valueOf as the valueOf of '4' " +
        " (threw " + Utils.referenceError(exception) +")",
        expect,
        result );

    return array;
}
