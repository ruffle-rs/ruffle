/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "switch";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The switch statement";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    SwitchTest( -2, 4 );
    SwitchTest( -1, 4 );
    SwitchTest( 0, 4 );
    SwitchTest( 1, 2 );
    SwitchTest( 2, 4 );
    SwitchTest( 4294967295, 4)

    function SwitchTest( input:uint, expect:uint ) {
        var result = 0;

        switch ( input ) {
            case -1:
                // not possible to reach this block with a uint
                result += 1;
                break;
            case 1:
                result += 2;
                break;
            default:
                result += 4;
                break;
        }

        array[item++] = Assert.expectEq(
            //SECTION,
            // TODO: REVIEW AS4 CONVERSION ISSUE
            "switch with negative value cases: input is " + input,
            expect,
            result );
    }
    return array;
}
