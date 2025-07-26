/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

/*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=598683
*
*/
//-----------------------------------------------------------------------------

import com.adobe.test.Utils;
import com.adobe.test.Assert;
// var SECTION = "593383";
// var VERSION = "";
// var TITLE   = "Testing various cases where we may optimize charCodeAt";
// var bug = "593383";

var testcases = getTestCases();

function getTestCases() {

    var actual:String;
    try
    {
        var xml:XML=new XML("<a><b:c xmlns:b=\"abc\"></d:c></a>") ;
        actual = xml.toXMLString();
    }
    catch(e)
    {
        actual = Utils.grabError(e, e.toString());
    }

    var expect:String= "Error #1085"; // kXMLUnterminatedElementTag

    var array = new Array();
    var index = 0;

    var s:String = "simple";

    // test optimizeIntCmpWithNumberCall logic with int charCodeAt(int)
    function testInt(outofbounds:int, inbounds:int)
    {
        var b1:Boolean;
        var b2:Boolean;
        var b3:Boolean;
        var b4:Boolean;
        var b5:Boolean;
        var b6:Boolean;

        // test out of bounds charCodeAt which will return NaN (or zero after an integer coerce)

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == 10);
        b2 = (10 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < 10);
        b4 = (s.charCodeAt(outofbounds) <= 10);
        b5 = (s.charCodeAt(outofbounds) > 10);
        b6 = (s.charCodeAt(outofbounds) >= 10);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == 10", b1, false);
        array[index++] = Assert.expectEq( "10 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < 10", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= 10", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > 10", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= 10", b5, false);

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == -10);
        b2 = (-10 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < -10);
        b4 = (s.charCodeAt(outofbounds) <= -10);
        b5 = (s.charCodeAt(outofbounds) > -10);
        b6 = (s.charCodeAt(outofbounds) >= -10);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == -10", b1, false);
        array[index++] = Assert.expectEq( "-10 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < -10", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= -10", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > -10", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= -10", b6, false);

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == 0);
        b2 = (0 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < 0);
        b4 = (s.charCodeAt(outofbounds) <= 0);
        b5 = (s.charCodeAt(outofbounds) > 0);
        b6 = (s.charCodeAt(outofbounds) >= 0);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == 0", b1, false);
        array[index++] = Assert.expectEq( "0 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < 0", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= 0", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > 0", b6, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= 0 THIS ONE", b5, false);

        // s.charCodeAt(1) == 105.  Using constant for compare
        b1 = (s.charCodeAt(inbounds) == 105);
        b2 = (105 == s.charCodeAt(inbounds));
        b3 = (s.charCodeAt(inbounds) < 105);
        b4 = (s.charCodeAt(inbounds) <= 105);
        b5 = (s.charCodeAt(inbounds) > 105);
        b6 = (s.charCodeAt(inbounds) >= 105);

        array[index++] = Assert.expectEq( "s.charCodeAt(1) == 105", b1, true);
        array[index++] = Assert.expectEq( "105 == s.charCodeAt(1)", b2, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) < 105", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) <= 105", b4, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) > 105", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) >= 105", b6, true);

        var i:int = s.charCodeAt(inbounds);
        b1 = (s.charCodeAt(inbounds) == i);
        b2 = (i == s.charCodeAt(inbounds));
        b3 = (s.charCodeAt(inbounds) < i);
        b4 = (s.charCodeAt(inbounds) <= i);
        b5 = (s.charCodeAt(inbounds) > i);
        b6 = (s.charCodeAt(inbounds) >= i);

        array[index++] = Assert.expectEq( "s.charCodeAt(1) == i, i=105", b1, true);
        array[index++] = Assert.expectEq( "i == s.charCodeAt(1),i=105", b2, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) < i,i=105", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) <= i,i=105", b4, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) > i,i=105", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) >= i,i=105", b6, true);
    }
    testInt(100,1);

    // test optimizeIntCmpWithNumberCall logic with int charCodeAt(uint)
    function testUint(outofbounds:uint, inbounds:uint)
    {
        var b1:Boolean;
        var b2:Boolean;
        var b3:Boolean;
        var b4:Boolean;
        var b5:Boolean;
        var b6:Boolean;

        // test out of bounds charCodeAt which will return NaN (or zero after an integer coerce)

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == 10);
        b2 = (10 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < 10);
        b4 = (s.charCodeAt(outofbounds) <= 10);
        b5 = (s.charCodeAt(outofbounds) > 10);
        b6 = (s.charCodeAt(outofbounds) >= 10);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == 10", b1, false);
        array[index++] = Assert.expectEq( "10 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < 10", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= 10", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > 10", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= 10", b5, false);

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == -10);
        b2 = (-10 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < -10);
        b4 = (s.charCodeAt(outofbounds) <= -10);
        b5 = (s.charCodeAt(outofbounds) > -10);
        b6 = (s.charCodeAt(outofbounds) >= -10);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == -10", b1, false);
        array[index++] = Assert.expectEq( "-10 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < -10", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= -10", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > -10", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= -10", b6, false);

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == 0);
        b2 = (0 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < 0);
        b4 = (s.charCodeAt(outofbounds) <= 0);
        b5 = (s.charCodeAt(outofbounds) > 0);
        b6 = (s.charCodeAt(outofbounds) >= 0);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == 0", b1, false);
        array[index++] = Assert.expectEq( "0 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < 0", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= 0", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > 0", b6, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= 0 THIS ONE", b5, false);

        // s.charCodeAt(1) == 105.  Using constant for compare
        b1 = (s.charCodeAt(inbounds) == 105);
        b2 = (105 == s.charCodeAt(inbounds));
        b3 = (s.charCodeAt(inbounds) < 105);
        b4 = (s.charCodeAt(inbounds) <= 105);
        b5 = (s.charCodeAt(inbounds) > 105);
        b6 = (s.charCodeAt(inbounds) >= 105);

        array[index++] = Assert.expectEq( "s.charCodeAt(1) == 105", b1, true);
        array[index++] = Assert.expectEq( "105 == s.charCodeAt(1)", b2, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) < 105", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) <= 105", b4, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) > 105", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) >= 105", b6, true);

        var i:int = s.charCodeAt(inbounds);
        b1 = (s.charCodeAt(inbounds) == i);
        b2 = (i == s.charCodeAt(inbounds));
        b3 = (s.charCodeAt(inbounds) < i);
        b4 = (s.charCodeAt(inbounds) <= i);
        b5 = (s.charCodeAt(inbounds) > i);
        b6 = (s.charCodeAt(inbounds) >= i);

        array[index++] = Assert.expectEq( "s.charCodeAt(1) == i, i=105", b1, true);
        array[index++] = Assert.expectEq( "i == s.charCodeAt(1),i=105", b2, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) < i,i=105", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) <= i,i=105", b4, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) > i,i=105", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) >= i,i=105", b6, true);
    }
    testUint(100,1);

    // test optimizeIntCmpWithNumberCall logic with int charCodeAt(double)
    function testDouble(outofbounds:Number, inbounds:Number)
    {
        var b1:Boolean;
        var b2:Boolean;
        var b3:Boolean;
        var b4:Boolean;
        var b5:Boolean;
        var b6:Boolean;

        // test out of bounds charCodeAt which will return NaN (or zero after an integer coerce)

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == 10);
        b2 = (10 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < 10);
        b4 = (s.charCodeAt(outofbounds) <= 10);
        b5 = (s.charCodeAt(outofbounds) > 10);
        b6 = (s.charCodeAt(outofbounds) >= 10);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == 10", b1, false);
        array[index++] = Assert.expectEq( "10 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < 10", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= 10", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > 10", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= 10", b5, false);

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == -10);
        b2 = (-10 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < -10);
        b4 = (s.charCodeAt(outofbounds) <= -10);
        b5 = (s.charCodeAt(outofbounds) > -10);
        b6 = (s.charCodeAt(outofbounds) >= -10);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == -10", b1, false);
        array[index++] = Assert.expectEq( "-10 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < -10", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= -10", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > -10", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= -10", b6, false);

        // NaN should not equal any of these
        b1 = (s.charCodeAt(outofbounds) == 0);
        b2 = (0 == s.charCodeAt(outofbounds));
        b3 = (s.charCodeAt(outofbounds) < 0);
        b4 = (s.charCodeAt(outofbounds) <= 0);
        b5 = (s.charCodeAt(outofbounds) > 0);
        b6 = (s.charCodeAt(outofbounds) >= 0);

        array[index++] = Assert.expectEq( "s.charCodeAt(100) == 0", b1, false);
        array[index++] = Assert.expectEq( "0 == s.charCodeAt(100)", b2, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) < 0", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) <= 0", b4, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) > 0", b6, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(100) >= 0 THIS ONE", b5, false);

        // s.charCodeAt(1) == 105.  Using constant for compare
        b1 = (s.charCodeAt(inbounds) == 105);
        b2 = (105 == s.charCodeAt(inbounds));
        b3 = (s.charCodeAt(inbounds) < 105);
        b4 = (s.charCodeAt(inbounds) <= 105);
        b5 = (s.charCodeAt(inbounds) > 105);
        b6 = (s.charCodeAt(inbounds) >= 105);

        array[index++] = Assert.expectEq( "s.charCodeAt(1) == 105", b1, true);
        array[index++] = Assert.expectEq( "105 == s.charCodeAt(1)", b2, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) < 105", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) <= 105", b4, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) > 105", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) >= 105", b6, true);

        var i:int = s.charCodeAt(inbounds);
        b1 = (s.charCodeAt(inbounds) == i);
        b2 = (i == s.charCodeAt(inbounds));
        b3 = (s.charCodeAt(inbounds) < i);
        b4 = (s.charCodeAt(inbounds) <= i);
        b5 = (s.charCodeAt(inbounds) > i);
        b6 = (s.charCodeAt(inbounds) >= i);

        array[index++] = Assert.expectEq( "s.charCodeAt(1) == i, i=105", b1, true);
        array[index++] = Assert.expectEq( "i == s.charCodeAt(1),i=105", b2, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) < i,i=105", b3, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) <= i,i=105", b4, true);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) > i,i=105", b5, false);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) >= i,i=105", b6, true);
    }
    testDouble(100.1,1.1);

    // Testing our code to swap a float returning charCodeAt to integer (Specializer)
    // passing in params to avoid constant folding
    function test2(i:int, ui:uint, d:Number, expectedInt:int, expectedDouble:Number)
    {
        var i1:int = s.charCodeAt(i);
        var i2:int = s.charCodeAt(ui);
        var i3:int = s.charCodeAt(d);
        var f1:Number = s.charCodeAt(i);
        var f2:Number = s.charCodeAt(ui);
        var f3:Number = s.charCodeAt(d);

        array[index++] = Assert.expectEq( "s.charCodeAt(1) == i, i=105", i1, expectedInt);
        array[index++] = Assert.expectEq( "i == s.charCodeAt(1),i=105", i2, expectedInt);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) < i,i=105", i3, expectedInt);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) <= i,i=105", f1, expectedDouble);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) > i,i=105", f2, expectedDouble);
        array[index++] = Assert.expectEq( "s.charCodeAt(1) >= i,i=105", f3, expectedDouble);

    }
    test2(1, 1, 1, 105, 105);
    test2(-1,-1,-1, 0, NaN);
    test2(100,100,100, 0, NaN);

    return array;
}
