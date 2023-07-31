/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Wraparound_Conversion";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "Wraparound_Conversion";       // Provide ECMA section title or a description
var BUGNUMBER = "";



///////////////////////////////////////////////////////////////
// add your tests here

/*
    Reference max / min vals:
    int.MAX_VALUE: 2147483647
    int.MIN_VALUE: -2147483648
    uint.MAX_VALUE: 4294967295
    uint.MIN_VALUE: 0
    Number.MAX_VALUE: 1.79769313486231e+308
    Number.MIN_VALUE: 4.9406564584124654e-324
*/

//simple wraparound tests

//int
var intNum:int;

var intMaxWrapAdd:int = int.MAX_VALUE + 1;
Assert.expectEq( "int.MAX_VALUE + 1 = -2147483648", -2147483648, intMaxWrapAdd );

intNum = int.MAX_VALUE;
intNum = intNum + 1;
Assert.expectEq( "int.MAX_VALUE + 1 = -2147483648", -2147483648, intNum );

intNum = int.MAX_VALUE;
intNum++;
Assert.expectEq( "increment int at int.MAX_VALUE", -2147483648, intNum );

var intMinWrapAdd:int = int.MIN_VALUE - 1;
Assert.expectEq( "int.MIN_VALUE - 1 = 2147483647", 2147483647, intMinWrapAdd );

intNum = int.MIN_VALUE;
intNum = intNum - 1;
Assert.expectEq( "int.MIN_VALUE - 1 = 2147483647", 2147483647, intNum );

intNum = int.MIN_VALUE;
intNum--;
Assert.expectEq( "decrement int at int.MIN_VALUE", 2147483647, intNum );

var intMaxWrapMult:int = int.MAX_VALUE * 2;
Assert.expectEq( "int.MAX_VALUE * 2 = -2", -2, intMaxWrapMult );

//uint
var uintNum:uint;

var uintMaxWrapAdd:uint = uint.MAX_VALUE + 1;
Assert.expectEq( "uint.MAX_VALUE + 1 = 0", 0, uintMaxWrapAdd );

uintNum = uint.MAX_VALUE;
uintNum = uintNum + 1;
Assert.expectEq( "uint.MAX_VALUE + 1 = 0", 0, uintNum );

uintNum = uint.MAX_VALUE;
uintNum++;
Assert.expectEq( "increment uint at uint.MAX_VALUE", 0, uintNum );

var uintMinWrapAdd:uint = uint.MIN_VALUE - 1;
Assert.expectEq( "uint.MIN_VALUE - 1 = 4294967295", 4294967295, uintMinWrapAdd );

uintNum = uint.MIN_VALUE;
uintNum = uintNum - 1;
Assert.expectEq( "uint.MIN_VALUE - 1 = 4294967295", 4294967295, uintNum );

uintNum = uint.MIN_VALUE;
uintNum--;
Assert.expectEq( "decrement uint at uint.MIN_VALUE", 4294967295, uintNum );

var uintMaxWrapMult:uint = uint.MAX_VALUE * 2;
Assert.expectEq( "uint.MAX_VALUE * 2 = 4294967294", 4294967294, uintMaxWrapMult );

//bitwise shift tests

intNum = int.MAX_VALUE;

intNum = intNum << 1;
Assert.expectEq( "int.MAX_VALUE << 1", -2, intNum );

uintNum = uint.MAX_VALUE;

// uint.MAX_VALUE << 1 (FFFFFFFF << 1 = 1FFFFFFFE) then to wraparound: 1FFFFFFFE % 100000000 which gives FFFFFFFE.
uintNum = uintNum << 1;
Assert.expectEq( "uint.MAX_VALUE << 1", 4294967294, uintNum );


//
////////////////////////////////////////////////////////////////

              // displays results.
              
              
              
              
              
              
