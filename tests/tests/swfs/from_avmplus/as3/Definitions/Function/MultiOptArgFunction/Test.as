/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import MultiOptArgFunction.*
import com.adobe.test.Assert;

class MultiOptArgFunctionClass {
    function returnArguments(s:String = "Str3", b:Boolean = true, n:Number = 30) {
        str = s;
        bool = b;
        num = n;
    }
}

function returnArgumentsNoPackage(s:String = "Str4", b:Boolean = false, n:Number = 40) {
    str = s;
    bool = b;
    num = n;
}


// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Body Parameter/Result Type";       // Provide ECMA section title or a description
var BUGNUMBER = "";



var TESTOBJ = new TestObj();
var TESTOBJ1 = new MultiOptArgFunctionClass();

var success = false;
TESTOBJ.returnArguments("String1");

if(str == "String1" && bool == true && num == 10)
{ success = true;}
else
{ success = false;}

Assert.expectEq( "TESTOBJ.returnArguments('String1');", true, success );


success = false;
TESTOBJ.returnArguments("String1",false);

if(str == "String1" && bool == false && num == 10)
{ success = true;}
else
{ success = false;}

Assert.expectEq( "TESTOBJ.returnArguments('String1',false)", true, success );


success = false;
TESTOBJ.returnArguments("String1",false,100);

if(str == "String1" && bool == false && num == 100)
{ success = true;}
else
{success = false;}

Assert.expectEq( "TESTOBJ.returnArguments('String1',false,100);", true, success );


success = false;
returnArguments("String2");

if(str == "String2" && bool == false && num == 20)
{success = true;}
else
{success = false;}

Assert.expectEq( "returnArguments('String2')", true, success );


success = false;
returnArguments("String2",true);

if(str == "String2" && bool == true && num == 20)
{success = true;}
else
{success = false;}

Assert.expectEq( "returnArguments('String2',true)", true, success );

success = false;
returnArguments("String2",true,100);

if(str == "String2" && bool == true && num == 100)
{success = true;}
else
{success = false;}

Assert.expectEq( "returnArguments('String2',true,100)", true, success );

success = false;
TESTOBJ1.returnArguments("String3");

if(str == "String3" && bool == true && num == 30)
{success = true;}
else
{success = false;}

Assert.expectEq( "TESTOBJ1.returnArguments('String3')", true, success );


success = false;
TESTOBJ1.returnArguments("String3",false);

if(str == "String3" && bool == false && num == 30)
{success = true;}
else
{success = false;}

Assert.expectEq( "TESTOBJ1.returnArguments('String3',false)", true, success );

success = false;
TESTOBJ1.returnArguments("String3",false,300);

if(str == "String3" && bool == false && num == 300)
{success = true;}
else
{success = false;}

Assert.expectEq( "TESTOBJ1.returnArguments('String3',false,300)", true, success );


success = false;
returnArgumentsNoPackage("String4");

if(str == "String4" && bool == false && num == 40)
{success = true;}
else
{success = false;}

Assert.expectEq( "returnArgumentsNoPackage('String4')", true, success );


success = false;
returnArgumentsNoPackage("String4",true);

if(str == "String4" && bool == true && num == 40)
{success = true;}
else
{success = false;}

Assert.expectEq( "returnArgumentsNoPackage('String4',true)", true, success );


success = false;
returnArgumentsNoPackage("String4",true,400);

if(str == "String4" && bool == true && num == 400)
{success = true;}
else
{success = false;}

Assert.expectEq( "returnArgumentsNoPackage('String4',true,400)", true, success );


              // displays results.
