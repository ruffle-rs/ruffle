/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "while-004";
//     var VERSION = "ECMA_2";
//     var TITLE   = "while statement";
    var BUGNUMBER="316725";


    var tc = 0;
    var testcases = new Array();

    DoWhile_1();
    DoWhile_2();
    DoWhile_3();
    DoWhile_4();
    DoWhile_5();


     function dowhile() {
        result = "pass";

        while (true) {
            return result;
            result = "fail: hit code after return statement";
            break;
        }
    }

   function DoWhile_1() {
        description = "return statement in a while block";

        result = dowhile();

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_1" + description,
            "pass",
            result );
    }

    function DoWhile_2() {
        var description = "while with a labeled continue statement";
        var result1 = "pass";
        var result2 = "fail: did not execute code after loop, but inside label";
        var i = 0;
        var j = 0;

        theloop:
            while( i++ < 10  ) {
                j++;
                continue theloop;
                result1 = "failed:  hit code after continue statement";
            }
        result2 = "pass";

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_2:  " +description + " - code inside the loop, before the continue should be executed ("+j+")",
            true,
            j == 10 );

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_2:  " +description +" - code after labeled continue should not be executed",
            "pass",
            result1 );

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_2:  " +description +" - code after loop but inside label should be executed",
            "pass",
            result2 );
    }

    function DoWhile_3() {
        var description = "while with a labeled break statement";
        var result1 = "pass";
        var result2 = "pass";
        var result3 = "fail: did not get to code after label";

        woohoo: {
            while( true ) {
                break woohoo;
                result1 = "fail: got to code after a break";
            }
            result2 = "fail: got to code outside of loop but inside label";
        }

        result3 = "pass";

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_3: " +description +" - verify break out of loop",
            "pass",
            result1 );


        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_3: " +description +" - verify break out of label",
            "pass",
            result2 );

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_3: " +description + " - verify correct exit from label",
            "pass",
            result3 );
    }


    function DoWhile_4() {
        var description = "labeled while with an unlabeled break";
        var result1 = "pass";
        var result2 = "pass";
        var result3 = "fail: did not evaluate statement after label";

        woohooboy: {
            while( true ) {
                break woohooboy;
                result1 = "fail: got to code after the break";
            }
            result2 = "fail: broke out of while, but not out of label";
        }
        result3 = "pass";

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_4: " +description +" - verify break out of while loop",
            "pass",
            result1 );

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_4: " +description + " - verify break out of label",
            "pass",
            result2 );

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_4: " +description +" - verify that statements after label are evaluated",
            "pass",
            result3 );
    }

    function DoWhile_5() {
        var description = "while with a labeled continue statement";
        var result1 = "pass";
        var result2 = "fail: did not execute code after loop, but inside label";
        var i = 0;
        var j = 0;

        theloop: {
            j++;
            while( i++ < 10  ) {
                continue;
                result1 = "failed:  hit code after continue statement";
            }
            result2 = "pass";
        }

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_5: " +description + " - continue should not execute statements above the loop",
            true,
            ( j == 1 ) );

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_5: " +description +" - code after labeled continue should not be executed",
            "pass",
            result1 );

        testcases[tc++] = Assert.expectEq(
            
            "DoWhile_5: " +description +" - code after loop but inside label should be executed",
            "pass",
            result2 );
    }

