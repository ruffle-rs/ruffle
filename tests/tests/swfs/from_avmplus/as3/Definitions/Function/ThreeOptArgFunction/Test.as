/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


import ThreeOptArgFunction.*
import com.adobe.test.Assert;

class ThreeOptArgFunctionClass {
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


// TODO: Review AS4 Conversion
//  These classes used to be external, but are now inside classes because of the change from include to import


    class TestObjInner{

      function returnArgumentsInner(s:String = "Str1", b:Boolean = true, n:Number = 10, ... rest) {
        str = s;
        bool = b;
        num = n;
      }

    }

     class TestObj extends TestObjInner {

         function returnArguments() { returnArgumentsInner("Str1", true, 10, 12); }

    }

        function returnArgumentsInner(s:String = "Str2", b:Boolean = false, n:Number = 20, ... rest) {

        str = s;
        bool = b;
        num = n;
    }


     function returnArguments() { returnArgumentsInner("Str2",false,20,true); }

 // END TODO



// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";  // Version of JavaScript or ECMA
// var TITLE   = "Function Body Parameter/Result Type";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var TESTOBJ = new TestObj();
var TESTOBJ1 = new ThreeOptArgFunctionClass();

var success = false;
TESTOBJ.returnArguments();

if(str == "Str1" && bool == true && num == 10)
{ success = true;}
else
{ success = false;}

Assert.expectEq( "TESTOBJ.returnArguments();", true, success );


success = false;
returnArguments();

if(str == "Str2" && bool == false && num == 20)
{ success = true;}
else
{ success = false;}

Assert.expectEq( "returnArguments();", true, success );


success = false;
TESTOBJ1.returnArguments();

if(str == "Str3" && bool == true && num == 30)
{ success = true;}
else
{ success = false;}

Assert.expectEq( "TESTOBJ1.returnArguments();", true, success );


success = false;
returnArgumentsNoPackage();

if(str == "Str4" && bool == false && num == 40)
{ success = true;}
else
{ success = false;}

Assert.expectEq( "returnArgumentsNoPackage();", true, success );


              // displays results.
