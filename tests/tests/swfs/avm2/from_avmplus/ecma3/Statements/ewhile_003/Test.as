/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "while-003";
//     var VERSION = "ECMA_2";
//     var TITLE   = "while statement";


    var testcases = getTestCases();
    
function getTestCases(){
    var array = new Array();
    var item = 0;

    DoWhile( new DoWhileObject(
                "while expression is true",
                true));

    DoWhile( new DoWhileObject(
             "while expression is 1",
             1));

    DoWhile( new DoWhileObject(
             "while expression is new Boolean(true)",
             new Boolean(true)));

    DoWhile( new DoWhileObject(
             "while expression is new Object()",
             new Object()));

    DoWhile( new DoWhileObject(
             "while expression is \"hi\"",
             "hi"));
/*
    DoWhile( new DoWhileObject(
             "while expression has a continue in it",
             "true",
             "if ( i == void 0 ) i = 0; result=\"pass\"; if ( ++i == 1 ) {continue;} else {break;} result=\"fail\";"
             ));
*/


    function DoWhileObject( d, e ) {
        this.description = d;
        this.whileExpression = e;
    }

    function DoWhile( object ) {
        result = "fail: statements in while block were not evaluated";

        while ( expression = object.whileExpression ) {
            result = "pass";
            break;
        }

        // verify that the while expression was evaluated
        array[item++] = Assert.expectEq(
            
            "verify that while expression was evaluated (should be "+
                object.whileExpression +")",
            "pass",
            (object.whileExpression == expression ||
               ( isNaN(object.whileExpression) && isNaN(expression) )
             ) ? "pass" : "fail" );

        array[item++] = Assert.expectEq(
            
            object.description,
            "pass",
            result );
    }
    return array;
}
