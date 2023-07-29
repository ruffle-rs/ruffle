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
    var BUGNUMBER= "315988";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    ONE = new Number(1);
    ZERO = new Number(0);
    var A = new String("A");
    var B = new String("B");
    TRUE = new Boolean( true );
    FALSE = new Boolean( false );
    UNDEFINED  = void 0;
    NULL = null;

    SwitchTest( ZERO, "0" ); // because Number will not return an object
    SwitchTest( NULL, "NULL" );
    SwitchTest( UNDEFINED, "UNDEFINED" );
    SwitchTest( FALSE, "false" );
    SwitchTest( false,  "false" );
    SwitchTest( 0,      "0" );

    SwitchTest ( TRUE, "true" );
    SwitchTest( 1,     "1" );
    SwitchTest( ONE,   "1" );
    SwitchTest( true,  "true" );

    SwitchTest( "a",   "a" );
    SwitchTest( A,     "A" );
    SwitchTest( "b",   "b" );
    SwitchTest( B,     "B" );

    SwitchTest( new Boolean( true ), "true" );
    SwitchTest( new Boolean(false ), "false" );
    SwitchTest( new String( "A" ),   "A" );
    SwitchTest( new Number( 0 ),     "0" );



    function SwitchTest( input, expect ) {
        var result = "";

        switch ( input ) {
            default:   result += "default"; break;
            case "a":  result += "a";       break;
            case "b":  result += "b";       break;
            case A:    result += "A";       break;
            case B:    result += "B";       break;
            //case new Boolean(true): result += "new TRUE";   break;
            //case new Boolean(false): result += "new FALSE"; break;
            case NULL: result += "NULL";    break;
            case UNDEFINED: result += "UNDEFINED"; break;
            case true: result += "true";    break;
            case false: result += "false";  break;
            case TRUE:  result += "TRUE";   break;
            case FALSE: result += "FALSE";  break;
            case 0:    result += "0";       break;
            case 1:    result += "1";       break;
            //case new Number(0) : result += "new ZERO";  break;
            //case new Number(1) : result += "new ONE";   break;
            case ONE:  result += "ONE";     break;
            case ZERO: result += "ZERO";    break;
        }

        array[item++] = Assert.expectEq(
            
            "switch with no breaks:  input is " + input,
            expect,
            result );
    }
    return array;
}
