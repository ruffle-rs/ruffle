/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "while-002";
//     var VERSION = "ECMA_2";
//     var TITLE   = "while statement";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    DoWhile( new DoWhileObject(
                "while expression is null",
                null,
                "result = \"fail: should not have evaluated statements in while block;break"
               ) );

    DoWhile( new DoWhileObject(
                "while expression is undefined",
                void 0,
                "result = \"fail: should not have evaluated statements in while block; break"
             ));

    DoWhile( new DoWhileObject(
             "while expression is 0",
             0,
             "result = \"fail: should not have evaluated statements in while block; break;"
             ));

    DoWhile( new DoWhileObject(
            "while expression is NaN",
            NaN,
            "result = \"fail: should not have evaluated statements in while block; break"
            ));


    function DoWhileObject( d, e, s ) {
        this.description = d;
        this.whileExpression = e;
        this.statements = s;
    }

    function DoWhile( object ) {
        result = "pass";

        while ( expression = object.whileExpression ) {
         object.statements ;
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
