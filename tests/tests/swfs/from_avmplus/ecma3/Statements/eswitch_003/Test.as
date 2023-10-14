/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "switch-003";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The switch statement";



    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    SwitchTest( "a", "abc" );
    SwitchTest( "b", "bc" );
    SwitchTest( "c", "c" );
    SwitchTest( "d", "*abc" );
    SwitchTest( "v", "*abc" );
    SwitchTest( "w", "w*abc" );
    SwitchTest( "x", "xw*abc" );
    SwitchTest( "y", "yxw*abc" );
    SwitchTest( "z", "zyxw*abc" );
//    SwitchTest( new java.lang.String("z"), "*abc" );



    function SwitchTest( input, expect ) {
        var result = "";

        switch ( input ) {
            case "z": result += "z";
            case "y": result += "y";
            case "x": result += "x";
            case "w": result += "w";
            default: result += "*";
            case "a": result += "a";
            case "b": result += "b";
            case "c": result += "c";
        }

        array[item++] = Assert.expectEq(
            
            "switch with no breaks:  input is " + input,
            expect,
            result );
    }
    return array;
}
